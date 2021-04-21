use anyhow::Result;
use async_std::{
  sync::Mutex,
  task::{self, block_on},
};
use crossbeam::atomic::AtomicCell;
use indexmap::IndexMap;
use itertools::Itertools;
use std::{
  collections::HashMap,
  sync::Arc,
  time::{Duration, Instant},
};
use three_commas_client::{DealsScope, ThreeCommasClient};
use three_commas_types::{Bot, BotStats, Deal, Pair};
use tracing::{span, Level};
use tracing_futures::Instrument;

use crate::decimal_gauge::Decimal;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CacheState {
  Stale(Instant),
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

  pub fn pairs(&self) -> &[Pair] {
    self.bot.pairs()
  }

  pub fn base_order_volume(&self) -> Decimal {
    self.bot.base_order_volume().into()
  }

  pub fn safety_order_volume(&self) -> Decimal {
    self.bot.safety_order_volume().into()
  }

  pub fn max_safety_orders(&self) -> usize {
    self.bot.max_safety_orders()
  }

  pub fn total_budget(&self) -> Decimal {
    self.bot.max_budget().into()
  }

  pub fn max_active_deals(&self) -> usize {
    self.bot.max_active_deals()
  }

  pub fn profits(&self) -> impl Iterator<Item = (&str, Decimal)> {
    self
      .stats
      .overall()
      .iter()
      .map(|(tok, value)| (tok, value.into()))
  }

  pub fn profits_in_usd(&self) -> Decimal {
    self.stats.overall_usd_profit().into()
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

  pub async fn new(client: &ThreeCommasClient) -> Result<Self> {
    let span = span!(Level::INFO, "3commas::scraper::cache::init");
    let data = fetch_data(client, IndexMap::new()).instrument(span).await?;

    Ok(Self {
      inner: Arc::new(Inner {
        client: client.clone(),
        state: AtomicCell::new(CacheState::Stale(Instant::now())),
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
        CacheState::Updating(_) => return,
        CacheState::Stale(v) => {
          if v.elapsed() >= Self::CACHE_DURATION {
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
    let span = span!(Level::INFO, "3commas::scraper::cache::update");
    let previous = { self.inner.cached.lock().await.as_ref().clone() };
    let data = fetch_data(&self.inner.client, previous.map)
      .instrument(span)
      .await
      .unwrap();

    let new = CacheState::Stale(Instant::now());
    if self
      .inner
      .state
      .compare_exchange(expected_state, new)
      .is_ok()
    {
      let mut guard = self.inner.cached.lock().await;
      *guard = Arc::new(data);
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

  let mut deals = HashMap::new();
  for (id, group) in &active_deals.into_iter().group_by(|d| d.bot_id()) {
    deals.insert(id, group.collect::<Vec<_>>());
  }

  let mut result = previous;
  if result.len() < bots.len() {
    result.reserve(bots.len() - result.len());
  }

  for bot in bots {
    let stats = client.bot_stats(&bot).await.map_err(|e| e.into_inner())?;
    let open_deals = deals.remove(&bot.id()).unwrap_or_default();

    let data = BotData {
      bot,
      stats,
      open_deals,
    };
    result.insert(data.bot.id(), Arc::new(data));
  }

  Ok(CachedData { map: result })
}
