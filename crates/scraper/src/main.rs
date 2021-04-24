mod cache;
mod decimal_gauge;
mod server_tracing;

use anyhow::Result;
use cache::Cache;
use clap::Clap;
use decimal_gauge::DecimalGaugeVec;
use prometheus::{
  core::{AtomicU64, GenericGaugeVec},
  Encoder, Opts, Registry, TextEncoder, TEXT_FORMAT,
};
use std::{env, sync::Arc};
use three_commas_client::ThreeCommasClient;
use tide::{Body, Request};
use tracing_subscriber::EnvFilter;

type U64GaugeVec = GenericGaugeVec<AtomicU64>;

#[derive(Clap, Debug, PartialEq)]
enum LogFormat {
  Pretty,
  Json,
}

#[derive(Clap, Debug)]
struct App {
  #[clap(arg_enum, env = "LOG_FORMAT", default_value = "pretty")]
  log_format: LogFormat,
}

#[derive(Clone)]
struct Gauges {
  base_order: DecimalGaugeVec,
  safety_order: DecimalGaugeVec,
  max_safety_orders: U64GaugeVec,
  max_deals: U64GaugeVec,
  total_budget: DecimalGaugeVec,
  profit: DecimalGaugeVec,
  profits_in_usd: DecimalGaugeVec,
  open_deals: U64GaugeVec,
}

#[derive(Clone)]
struct AppState {
  cache: Cache,
  registry: Registry,
  gauges: Arc<Gauges>,
}

fn bot_opts(name: &str, help: &str) -> Opts {
  Opts::new(name, help)
    .namespace("three_commas")
    .subsystem("bots")
}

#[async_std::main]
async fn main() -> Result<()> {
  let app = App::parse();

  match app.log_format {
    LogFormat::Pretty => {
      tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_max_level(tracing::Level::INFO)
        .init();
    }
    LogFormat::Json => {
      tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();
    }
  }

  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .with_max_level(tracing::Level::INFO)
    .init();

  let api_key = env::var("API_KEY").unwrap_or_else(|_| panic!("No API_KEY set"));
  let api_secret = env::var("API_SECRET").unwrap_or_else(|_| panic!("No API_SECRET set"));

  let base_order = DecimalGaugeVec::new(
    bot_opts("base_order_volume", "Bot base order volume"),
    &["bot_id", "account_id", "currency"],
  )?;
  let safety_order = DecimalGaugeVec::new(
    bot_opts("safety_order_volume", "Bot initial safety order volume"),
    &["bot_id", "account_id", "currency"],
  )?;
  let max_safety_orders = U64GaugeVec::new(
    bot_opts("max_safety_orders", "Bot max safety orders"),
    &["bot_id", "account_id"],
  )?;
  let max_deals = U64GaugeVec::new(
    bot_opts("max_active_deals", "Bot max concurrent deals"),
    &["bot_id", "account_id"],
  )?;
  let total_budget = DecimalGaugeVec::new(
    bot_opts("total_budget", "Bot total budget"),
    &["bot_id", "account_id", "currency"],
  )?;
  let profit = DecimalGaugeVec::new(
    bot_opts("profit", "Bot profit"),
    &["bot_id", "account_id", "currency"],
  )?;
  let profits_in_usd = DecimalGaugeVec::new(
    bot_opts("profits_in_usd", "Bot profit converted to USD"),
    &["bot_id", "account_id"],
  )?;
  let open_deals = U64GaugeVec::new(
    bot_opts("open_deals", "Bot open deals"),
    &["bot_id", "account_id"],
  )?;

  let registry = Registry::new();
  registry.register(Box::new(base_order.clone()))?;
  registry.register(Box::new(safety_order.clone()))?;
  registry.register(Box::new(max_safety_orders.clone()))?;
  registry.register(Box::new(max_deals.clone()))?;
  registry.register(Box::new(total_budget.clone()))?;
  registry.register(Box::new(profit.clone()))?;
  registry.register(Box::new(profits_in_usd.clone()))?;
  registry.register(Box::new(open_deals.clone()))?;

  let client = ThreeCommasClient::new(api_key, api_secret);
  let cache = Cache::new(&client).await?;

  let gauges = Arc::new(Gauges {
    base_order,
    safety_order,
    max_safety_orders,
    max_deals,
    total_budget,
    profit,
    profits_in_usd,
    open_deals,
  });
  let mut app = tide::with_state(AppState {
    cache,
    registry,
    gauges,
  });
  app.with(server_tracing::TracingMiddlware);
  app.at("/metrics").get(get_metrics);

  app.listen("0.0.0.0:8080").await?;
  Ok(())
}

async fn get_metrics(req: Request<AppState>) -> tide::Result<Body> {
  let state = req.state();
  let gauges = &*state.gauges;

  for bot in state.cache.iter() {
    let bot_id = bot.id().to_string();
    let account_id = bot.account_id().to_string();
    let currency = bot.pairs().first().unwrap().quote();

    gauges
      .base_order
      .with_label_values(&[&*bot_id, &*account_id, &*currency])
      .set(bot.base_order_volume());

    gauges
      .safety_order
      .with_label_values(&[&*bot_id, &*account_id, &*currency])
      .set(bot.safety_order_volume());

    gauges
      .max_safety_orders
      .with_label_values(&[&*bot_id, &*account_id])
      .set(bot.max_safety_orders() as u64);

    gauges
      .max_deals
      .with_label_values(&[&*bot_id, &*account_id])
      .set(bot.max_active_deals() as u64);

    gauges
      .total_budget
      .with_label_values(&[&*bot_id, &*account_id, &*currency])
      .set(bot.total_budget());

    for (tok, value) in bot.profits() {
      gauges
        .profit
        .with_label_values(&[&*bot_id, &*account_id, tok])
        .set(value);
    }

    gauges
      .profits_in_usd
      .with_label_values(&[&*bot_id, &*account_id])
      .set(bot.profits_in_usd());

    gauges
      .open_deals
      .with_label_values(&[&*bot_id, &*account_id])
      .set(bot.open_deals() as u64);
  }

  let mut buffer = Vec::new();
  let encoder = TextEncoder::new();

  // Gather the metrics.
  let metric_families = state.registry.gather();

  // Encode them.
  encoder.encode(&metric_families, &mut buffer)?;

  let mut body = Body::from_bytes(buffer);
  body.set_mime(TEXT_FORMAT);
  Ok(body)
}
