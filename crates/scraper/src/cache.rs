mod cached;

pub use cached::Cached;

use crate::error::IntoReport;
use async_std::{
  sync::Mutex,
  task::{self, block_on},
};
use chrono::{Duration, Utc};
use color_eyre::Result;
use crossbeam::atomic::AtomicCell;
use futures::{future, TryStreamExt};
use im::OrdMap;
use rust_decimal::Decimal;
use std::{
  collections::{HashMap, HashSet},
  sync::Arc,
  time::Instant,
};
use three_commas_client::{AccountId, Bot, BotStats, Deal, DealsScope, Pair, ThreeCommasClient};
use tracing::{event, span, Level};
use tracing_futures::Instrument;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CacheState {
  Stale(Instant, Duration),
  Updating(Instant),
}

pub struct BotData {
  bot: Cached<Bot>,
  stats: Cached<BotStats>,
  deals: OrdMap<usize, Cached<Deal>>,
  open_deals: usize,
}

impl BotData {
  pub fn id(&self) -> usize {
    self.bot.id()
  }

  pub fn name(&self) -> &str {
    self.bot.name()
  }

  pub fn account_id(&self) -> usize {
    self.bot.account_id()
  }

  pub fn is_enabled(&self) -> bool {
    self.bot.is_enabled()
  }

  pub fn pairs(&self) -> &[Pair] {
    self.bot.pairs()
  }

  pub fn currency(&self) -> &str {
    self.pairs().first().unwrap().quote()
  }

  pub fn base_order_volume(&self) -> Decimal {
    self.bot.base_order_volume()
  }

  pub fn safety_order_volume(&self) -> Decimal {
    self.bot.safety_order_volume()
  }

  pub fn max_safety_orders(&self) -> usize {
    self.bot.max_safety_orders()
  }

  pub fn total_budget(&self) -> Decimal {
    self.bot.max_budget()
  }

  pub fn max_active_deals(&self) -> usize {
    self.bot.max_active_deals()
  }

  pub fn profits(&self) -> impl Iterator<Item = (&str, Decimal)> {
    self.stats.overall().iter()
  }

  pub fn deals(&self) -> impl Iterator<Item = &Cached<Deal>> {
    self.deals.values()
  }

  pub fn profits_in_usd(&self) -> Decimal {
    self.stats.overall_usd_profit()
  }

  pub fn open_deals(&self) -> usize {
    self.open_deals
  }
}

pub struct AccountData {
  account_id: AccountId,
  name: String,
  btc_amount: Decimal,
  usd_amount: Decimal,
}

impl AccountData {
  pub fn id(&self) -> AccountId {
    self.account_id
  }

  pub fn name(&self) -> &str {
    &*self.name
  }

  pub fn btc_amount(&self) -> Decimal {
    self.btc_amount
  }

  pub fn usd_amount(&self) -> Decimal {
    self.usd_amount
  }
}

impl From<three_commas_client::Account> for AccountData {
  fn from(value: three_commas_client::Account) -> Self {
    Self {
      account_id: value.id,
      name: value.name,
      btc_amount: value.btc_amount,
      usd_amount: value.usd_amount,
    }
  }
}

#[derive(Default)]
pub struct Data {
  bots: OrdMap<usize, Cached<BotData>>,
  accounts: OrdMap<AccountId, Cached<AccountData>>,
}

impl Data {
  pub fn bots(&self) -> impl Iterator<Item = &Cached<BotData>> {
    self.bots.values()
  }

  pub fn accounts(&self) -> impl Iterator<Item = &Cached<AccountData>> {
    self.accounts.values()
  }
}

struct Inner {
  client: ThreeCommasClient,
  state: AtomicCell<CacheState>,
  cached: Mutex<Cached<Data>>,
}

#[derive(Clone)]
pub struct Cache {
  inner: Arc<Inner>,
}

impl Cache {
  fn cache_duration() -> Duration {
    Duration::seconds(30 /* 30 seconds */)
  }
  fn error_duration() -> Duration {
    Duration::minutes(15 /* 15 minutes */)
  }
  fn update_timeout_duration() -> Duration {
    Duration::minutes(15 /* 15 minutes */)
  }

  pub async fn new(client: &ThreeCommasClient) -> Result<Self> {
    let span = span!(target: "3commas::cache", Level::INFO, "3commas::scraper::cache::init");
    let data = fetch_data(client, Cached::new(Default::default()))
      .instrument(span)
      .await?;

    Ok(Self {
      inner: Arc::new(Inner {
        client: client.clone(),
        state: AtomicCell::new(CacheState::Stale(Instant::now(), Self::cache_duration())),
        cached: Mutex::new(data),
      }),
    })
  }

  pub fn data(&self) -> Cached<Data> {
    self.maybe_start_update();

    if let Some(guard) = self.inner.cached.try_lock() {
      guard.clone()
    } else {
      block_on(async { self.inner.cached.lock().await.clone() })
    }
  }

  fn maybe_start_update(&self) {
    loop {
      let state = self.inner.state.load();

      match state {
        CacheState::Updating(v) => {
          let elapsed = Duration::from_std(v.elapsed()).unwrap();
          event!(target: "3commas::cache", Level::INFO, elapsed = ?elapsed.to_std().unwrap(), "cache updating - time elapsed");
          if elapsed > Self::update_timeout_duration() * 2 {
            event!(
              target: "3commas::cache",
              Level::INFO,
              "cache updating - resetting status due to long elapsed time"
            );
            self
              .inner
              .state
              .store(CacheState::Stale(Instant::now(), Duration::zero()));
          }
          return;
        }

        CacheState::Stale(v, wait_time) => {
          let elapsed = Duration::from_std(v.elapsed()).unwrap();
          event!(target: "3commas::cache", Level::INFO, elapsed = ?elapsed.to_std().unwrap(), wait_time = ?wait_time.to_std().unwrap(), "cache stale elapsed");
          if elapsed >= wait_time {
            let new = CacheState::Updating(Instant::now());
            if self.inner.state.compare_exchange(state, new).is_ok() {
              let clone = self.clone();
              task::spawn(async move { clone.update(new).await });
            }

            // else: retry loop
          } else {
            return;
          }
        }
      }
    }
  }

  async fn update(&self, expected_state: CacheState) {
    let span = span!(target: "3commas::cache", Level::INFO, "3commas::scraper::cache::update");
    span.in_scope(|| event!(target: "3commas::cache", Level::INFO, "updating cache"));
    let previous = { self.inner.cached.lock().await.clone() };
    let data = async_std::future::timeout(
      Self::update_timeout_duration().to_std().unwrap(),
      fetch_data(&self.inner.client, previous),
    )
    .instrument(span.clone())
    .await;

    let wait_time = match &data {
      Err(_) => {
        span.in_scope(|| event!(target: "3commas::cache", Level::WARN, "update timed out"));
        Self::cache_duration()
      }
      Ok(Err(e)) => {
        span.in_scope(
          || event!(target: "3commas::cache", Level::WARN, error = ?e, "failed to update cache"),
        );
        Self::error_duration()
      }
      Ok(Ok(_)) => {
        span.in_scope(|| event!(target: "3commas::cache", Level::INFO, "updated cache data"));
        Self::cache_duration()
      }
    };

    let new = CacheState::Stale(Instant::now(), wait_time);
    if self
      .inner
      .state
      .compare_exchange(expected_state, new)
      .is_ok()
    {
      if let Ok(Ok(data)) = data {
        let mut guard = self.inner.cached.lock().await;
        *guard = data;
      }
    }
  }
}

async fn fetch_deals(client: &ThreeCommasClient) -> Result<Vec<Cached<Deal>>> {
  // first - get all deals started within the last year
  let one_year_ago = Utc::now() - Duration::days(365);
  let mut all_deals: Vec<Cached<Deal>> = client
    .deals()
    .limit(1000)
    .map_ok(Cached::new)
    .try_take_while(|d| future::ready(Ok(d.created_at() > one_year_ago)))
    .try_collect()
    .await
    .map_err(|e| e.into_inner().into_report())?;

  let seen_deals = all_deals.iter().map(|d| d.id()).collect::<HashSet<usize>>();

  let active_deals: Vec<Cached<Deal>> = client
    .deals()
    .scope(Some(DealsScope::Active))
    .limit(100)
    .map_ok(Cached::new)
    .try_collect()
    .await
    .map_err(|e| e.into_inner().into_report())?;

  let unseen_active_deals = active_deals
    .into_iter()
    .filter(|d| !seen_deals.contains(&d.id()));
  all_deals.extend(unseen_active_deals);

  Ok(all_deals)
}

async fn fetch_data(client: &ThreeCommasClient, previous: Cached<Data>) -> Result<Cached<Data>> {
  let accounts = client
    .accounts()
    .await
    .map_err(|e| e.into_inner().into_report())?;

  let summary_account = client
    .account(AccountId::Summary)
    .await
    .map_err(|e| e.into_inner().into_report())?;

  let mut accounts_builder = previous.accounts.clone();
  accounts_builder.insert(AccountId::Summary, Cached::new(summary_account.into()));
  for account in accounts {
    accounts_builder.insert(account.id, Cached::new(account.into()));
  }

  let bots = {
    let bots = client
      .bots()
      .await
      .map_err(|e| e.into_inner().into_report())?;
    let now = Utc::now();
    bots
      .into_iter()
      .map(|b| Cached::new_at(b, now))
      .collect::<Vec<_>>()
  };
  let all_deals = fetch_deals(client).await?;

  event!(target: "3commas::cache", Level::DEBUG, deals = all_deals.len(), "fetched a total of {} deals", all_deals.len());
  let mut deals: HashMap<usize, Vec<Cached<Deal>>> = HashMap::with_capacity(bots.len());
  for deal in all_deals {
    deals.entry(deal.bot_id()).or_default().push(deal);
  }

  let mut bot_builder = previous.bots.clone();
  for bot in bots {
    let stats = {
      let stats = client
        .bot_stats(&bot)
        .await
        .map_err(|e| e.into_inner().into_report())?;
      Cached::new(stats)
    };
    let bot_deals = deals.remove(&bot.id()).unwrap_or_default();
    let open_deals = bot_deals.iter().filter(|d| d.is_active()).count();
    event!(
      target: "3commas::cache",
      Level::DEBUG,
      deals = bot_deals.len(),
      open_deals = open_deals,
      bot = bot.id(),
      "bot deals"
    );

    let mut deals = bot_builder
      .get(&bot.id())
      .map(|b| b.deals.clone())
      .unwrap_or_default();
    deals.extend(bot_deals.into_iter().map(|d| (d.id(), d)));

    let data = BotData {
      bot,
      stats,
      deals,
      open_deals,
    };
    bot_builder.insert(data.bot.id(), Cached::new(data));
  }

  event!(
    target: "3commas::cache",
    Level::DEBUG,
    count = deals.len(),
    "deals not connected to a known bot"
  );
  Ok(Cached::new(Data {
    bots: bot_builder,
    accounts: accounts_builder,
  }))
}
