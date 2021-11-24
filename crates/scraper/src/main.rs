mod cache;
mod gauges;
mod metric;
mod server_tracing;
mod telegraf;

use anyhow::Result;
use cache::{Cache, Data};
use clap::{ArgEnum, ArgSettings, Parser};
use metric::{BotGauge, BotLabels};
use prometheus::{Encoder, Registry, TextEncoder, TEXT_FORMAT};
use rust_decimal::Decimal;
use std::{
  fmt::{self, Write},
  sync::Arc,
};
use three_commas_client::ThreeCommasClient;
use tide::{Body, Request, Response};
use tracing_subscriber::EnvFilter;

#[derive(ArgEnum, Debug, PartialEq, Clone, Copy)]
enum LogFormat {
  Pretty,
  Json,
}

#[derive(Parser, Debug)]
struct App {
  /// Log output format
  #[clap(
    arg_enum,
    long = "log-format",
    short = 'f',
    env = "LOG_FORMAT",
    default_value = "pretty"
  )]
  log_format: LogFormat,

  /// 3commas API key
  #[clap(env = "API_KEY", long = "api-key", short = 'k', setting = ArgSettings::HideEnvValues)]
  api_key: String,

  /// 3commas API secret
  #[clap(env = "API_SECRET", long = "api-secret", short = 's', setting = ArgSettings::HideEnvValues)]
  api_secret: String,

  /// Port number
  #[clap(env = "PORT", long = "port", short = 'p', default_value = "8080")]
  port: usize,
}

#[derive(Clone)]
struct Gauges {
  base_order: BotGauge<Decimal, 0>,
  safety_order: BotGauge<Decimal, 0>,
  max_safety_orders: BotGauge<usize, 0>,
  max_deals: BotGauge<usize, 0>,
  total_budget: BotGauge<Decimal, 0>,
  current_budget: BotGauge<Decimal, 0>,
  profit: BotGauge<Decimal, 0>,
  profits_in_usd: BotGauge<Decimal, 0>,
  open_deals: BotGauge<usize, 0>,
  enabled: BotGauge<bool, 0>,
}

#[derive(Clone)]
struct AppState {
  cache: Cache,
  registry: Registry,
  gauges: Arc<Gauges>,
}

#[async_std::main]
async fn main() -> Result<()> {
  let app = App::parse();

  let filter = EnvFilter::from_default_env()
    // Set the base level when not matched by other directives to INFO.
    .add_directive(tracing::Level::INFO.into());

  match app.log_format {
    LogFormat::Pretty => {
      tracing_subscriber::fmt().with_env_filter(filter).init();
    }
    LogFormat::Json => {
      tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .with_current_span(false)
        .with_span_list(false)
        .init();
    }
  }

  let api_key = app.api_key;
  let api_secret = app.api_secret;
  let port = app.port;

  let base_order = BotGauge::new("base_order_volume", "Bot base order volume")?;
  let safety_order = BotGauge::new("safety_order_volume", "Bot initial safety order volume")?;
  let max_safety_orders = BotGauge::new("max_safety_orders", "Bot max safety orders")?;
  let max_deals = BotGauge::new("max_active_deals", "Bot max concurrent deals")?;
  let total_budget = BotGauge::new("total_budget", "Bot total budget")?;
  let current_budget = BotGauge::new("current_budget", "Bot current budget")?;
  let profit = BotGauge::new("profit", "Bot profit")?;
  let profits_in_usd = BotGauge::new("profits_in_usd", "Bot profit converted to USD")?;
  let open_deals = BotGauge::new("open_deals", "Bot open deals")?;
  let enabled = BotGauge::new("enabled", "Bot enabled")?;

  let registry = Registry::new();
  base_order.register(&registry)?;
  safety_order.register(&registry)?;
  max_safety_orders.register(&registry)?;
  max_deals.register(&registry)?;
  total_budget.register(&registry)?;
  current_budget.register(&registry)?;
  profit.register(&registry)?;
  profits_in_usd.register(&registry)?;
  open_deals.register(&registry)?;
  enabled.register(&registry)?;

  let client = ThreeCommasClient::new(api_key, api_secret);
  let cache = Cache::new(&client).await?;

  let gauges = Arc::new(Gauges {
    base_order,
    safety_order,
    max_safety_orders,
    max_deals,
    total_budget,
    current_budget,
    profit,
    profits_in_usd,
    open_deals,
    enabled,
  });
  let mut app = tide::with_state(AppState {
    cache,
    registry,
    gauges,
  });
  app.with(server_tracing::TracingMiddlware);
  app.at("/health").get(get_health);
  app.at("/metrics").get(get_metrics);
  app.at("/telegraf").get(get_telegraf_metrics);

  app.listen(format!("0.0.0.0:{}", port)).await?;
  Ok(())
}

async fn get_health(_: Request<AppState>) -> tide::Result {
  Ok(Response::new(200))
}

async fn get_metrics(req: Request<AppState>) -> tide::Result<Body> {
  let state = req.state();
  let gauges = &*state.gauges;

  let data = state.cache.data();
  for bot in data.iter() {
    let labels = BotLabels::new(bot);
    gauges.enabled.set(&labels, bot.is_enabled());
    gauges.base_order.set(&labels, bot.base_order_volume());
    gauges.safety_order.set(&labels, bot.safety_order_volume());
    gauges
      .max_safety_orders
      .set(&labels, bot.max_safety_orders());
    gauges.max_deals.set(&labels, bot.max_active_deals());
    gauges.total_budget.set(&labels, bot.total_budget());
    gauges.profits_in_usd.set(&labels, bot.profits_in_usd());
    gauges.open_deals.set(&labels, bot.open_deals());

    gauges.current_budget.set(
      &labels,
      if bot.is_enabled() {
        bot.total_budget()
      } else {
        0.into()
      },
    );

    if let Some((_, profits)) = bot.profits().find(|(tok, _)| *tok == bot.currency()) {
      gauges.profit.set(&labels, profits);
    }
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

struct TelegrafWriter<'a>(&'a Data);

impl<'a> fmt::Display for TelegrafWriter<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for bot in self.0.iter() {
      telegraf::write_metrics_for_bot(bot, f)?;
    }

    Ok(())
  }
}

async fn get_telegraf_metrics(req: Request<AppState>) -> tide::Result<Body> {
  let data = req.state().cache.data();

  let mut buffer: String = String::new();
  write!(&mut buffer, "{}", TelegrafWriter(&*data))?;

  let mut body = Body::from_string(buffer);
  body.set_mime("plain/text");
  Ok(body)
}
