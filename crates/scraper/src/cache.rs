use anyhow::Result;
use async_std::{
  sync::Mutex,
  task::{self, block_on},
};
use crossbeam::atomic::AtomicCell;
use indexmap::IndexMap;
use rust_decimal::Decimal;
use std::{
  collections::HashMap,
  sync::Arc,
  time::{Duration, Instant},
};
use three_commas_client::{DealsScope, ThreeCommasClient};
use three_commas_types::{Bot, BotStats, Deal, Pair};
use tracing::{event, span, Level};
use tracing_futures::Instrument;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CacheState {
  Stale(Instant, Duration),
  Updating(Instant),
}

pub struct BotData {
  bot: Bot,
  stats: BotStats,
  open_deals: Vec<Deal>,
}

impl BotData {
  pub fn id(&self) -> usize {
    self.bot.id()
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

  pub fn profits_in_usd(&self) -> Decimal {
    self.stats.overall_usd_profit()
  }

  pub fn open_deals(&self) -> usize {
    self.open_deals.len()
  }
}

#[derive(Clone)]
pub struct CachedData {
  map: IndexMap<usize, Arc<BotData>>,
}

struct Inner {
  client: ThreeCommasClient,
  state: AtomicCell<CacheState>,
  cached: Mutex<Arc<CachedData>>,
}

#[derive(Clone)]
pub struct Cache {
  inner: Arc<Inner>,
}

impl Cache {
  const CACHE_DURATION: Duration = Duration::from_secs(60 * 2 /* 2 minutes */);
  const ERROR_DURATION: Duration = Duration::from_secs(60 * 15 /* 15 minutes */);
  const UPDATE_TIMEOUT_DURATION: Duration = Duration::from_secs(60 * 15 /* 15 minutes */);

  pub async fn new(client: &ThreeCommasClient) -> Result<Self> {
    let span = span!(target: "3commas::cache", Level::INFO, "3commas::scraper::cache::init");
    let data = fetch_data(client, IndexMap::new()).instrument(span).await?;

    Ok(Self {
      inner: Arc::new(Inner {
        client: client.clone(),
        state: AtomicCell::new(CacheState::Stale(Instant::now(), Self::CACHE_DURATION)),
        cached: Mutex::new(Arc::new(data)),
      }),
    })
  }

  pub fn iter(&self) -> impl Iterator<Item = Arc<BotData>> {
    self.maybe_start_update();

    let cached = {
      if let Some(guard) = self.inner.cached.try_lock() {
        guard.clone()
      } else {
        block_on(async { self.inner.cached.lock().await.clone() })
      }
    };

    cached
      .map
      .iter()
      .map(|(_, b)| b.clone())
      .collect::<Vec<_>>()
      .into_iter()
  }

  fn maybe_start_update(&self) {
    loop {
      let state = self.inner.state.load();
      match state {
        CacheState::Updating(v) => {
          let elapsed = v.elapsed();
          event!(target: "3commas::cache", Level::INFO, ?elapsed, "cache updating - time elapsed");
          if elapsed > Self::UPDATE_TIMEOUT_DURATION * 2 {
            event!(
              target: "3commas::cache",
              Level::INFO,
              "cache updating - resetting status due to long elapsed time"
            );
            self
              .inner
              .state
              .store(CacheState::Stale(Instant::now(), Duration::ZERO));
          }
          return;
        }
        CacheState::Stale(v, wait_time) => {
          let elapsed = v.elapsed();
          event!(target: "3commas::cache", Level::INFO, ?elapsed, ?wait_time, "cache stale elapsed");
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
    let previous = { self.inner.cached.lock().await.as_ref().clone() };
    let data = async_std::future::timeout(
      Self::UPDATE_TIMEOUT_DURATION,
      fetch_data(&self.inner.client, previous.map),
    )
    .instrument(span.clone())
    .await;

    let wait_time = match &data {
      Err(_) => {
        span.in_scope(|| event!(target: "3commas::cache", Level::WARN, "update timed out"));
        Self::CACHE_DURATION
      }
      Ok(Err(e)) => {
        span.in_scope(
          || event!(target: "3commas::cache", Level::WARN, error = ?e, "failed to update cache"),
        );
        Self::ERROR_DURATION
      }
      Ok(Ok(_)) => {
        span.in_scope(|| event!(target: "3commas::cache", Level::INFO, "updated cache data"));
        Self::CACHE_DURATION
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
        *guard = Arc::new(data);
      }
    }
  }
}

async fn fetch_data(
  client: &ThreeCommasClient,
  previous: IndexMap<usize, Arc<BotData>>,
) -> Result<CachedData> {
  let bots = client.bots().await.map_err(|e| e.into_inner())?;
  let active_deals = client
    .deals(DealsScope::Active)
    .await
    .map_err(|e| e.into_inner())?;

  event!(target: "3commas::cache", Level::DEBUG, deals = active_deals.len(), "got open deals");
  let mut deals: HashMap<usize, Vec<Deal>> = HashMap::new();
  for deal in active_deals {
    deals.entry(deal.bot_id()).or_default().push(deal);
  }

  let mut result = previous;
  if result.len() < bots.len() {
    result.reserve(bots.len() - result.len());
  }

  for bot in bots {
    let stats = client.bot_stats(&bot).await.map_err(|e| e.into_inner())?;
    let open_deals = deals.remove(&bot.id()).unwrap_or_default();
    event!(
      target: "3commas::cache",
      Level::DEBUG,
      deals = open_deals.len(),
      bot = bot.id(),
      "bot open deals"
    );

    event!(
      Level::DEBUG,
      bot = bot.id(),
      open_deals = open_deals.len(),
      "open deals on bot"
    );
    let data = BotData {
      bot,
      stats,
      open_deals,
    };
    result.insert(data.bot.id(), Arc::new(data));
  }

  event!(
    target: "3commas::cache",
    Level::DEBUG,
    count = deals.len(),
    "deals not connected to a known bot"
  );
  Ok(CachedData { map: result })
}
