use chrono::{DateTime, Utc};
use core::str;
use rust_decimal::Decimal;
use std::{
  fmt::{self, Write},
  iter::{Copied, Zip},
  slice::Iter,
};

use crate::cache::{BotData, Cached};

pub trait MetricValue {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

macro_rules! impl_metric_value {
  (<$lt:lifetime> $ty:ty, $fmt_trait:path) => {
    impl<$lt> MetricValue for $ty {
      #[inline]
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as $fmt_trait>::fmt(&self, f)
      }
    }
  };

  ($ty:ty, $fmt_trait:path) => {
    impl MetricValue for $ty {
      #[inline]
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as $fmt_trait>::fmt(&self, f)
      }
    }
  };
}

impl_metric_value!(<'a> &'a str, fmt::Debug);
impl_metric_value!(String, fmt::Debug);
impl_metric_value!(usize, fmt::Display);
impl_metric_value!(isize, fmt::Display);
impl_metric_value!(u64, fmt::Display);
impl_metric_value!(i64, fmt::Display);
impl_metric_value!(Decimal, fmt::Display);
impl_metric_value!(bool, fmt::Display);

pub trait TagSet<'a> {
  type Iter: Iterator<Item = (&'a str, &'a str)>;

  fn iter(&self) -> Self::Iter;
}
pub trait FieldSet<'a> {
  type Iter: Iterator<Item = (&'a str, &'a dyn MetricValue)>;

  fn iter(&self) -> Self::Iter;
}

struct Tags<'a, const L: usize> {
  names: &'a [&'a str; L],
  values: &'a [&'a str; L],
}

impl<'a, const L: usize> Tags<'a, L> {
  pub fn new(names: &'a [&'a str; L], values: &'a [&'a str; L]) -> Self {
    Self { names, values }
  }
}

impl<'a, const L: usize> TagSet<'a> for Tags<'a, L> {
  type Iter = Zip<Copied<Iter<'a, &'a str>>, Copied<Iter<'a, &'a str>>>;

  fn iter(&self) -> Self::Iter {
    let names = self.names.iter().copied();
    let values = self.values.iter().copied();

    names.zip(values)
  }
}

struct Fields<'a, const L: usize> {
  names: &'a [&'a str; L],
  values: &'a [&'a dyn MetricValue; L],
}

impl<'a, const L: usize> Fields<'a, L> {
  pub fn new(names: &'a [&'a str; L], values: &'a [&'a dyn MetricValue; L]) -> Self {
    Self { names, values }
  }
}

impl<'a, const L: usize> FieldSet<'a> for Fields<'a, L> {
  type Iter = Zip<Copied<Iter<'a, &'a str>>, Copied<Iter<'a, &'a dyn MetricValue>>>;

  fn iter(&self) -> Self::Iter {
    let names = self.names.iter().copied();
    let values = self.values.iter().copied();

    names.zip(values)
  }
}

const BOT_TAG_NAMES: &[&str; 4] = &["botId", "accountId", "quoteCurrency", "botName"];
const BOT_TAG_NAMES_WITH_CURRENCY: &[&str; 5] = &[
  "botId",
  "accountId",
  "quoteCurrency",
  "botName",
  "baseCurrency",
];
const DEAL_TAG_NAMES: &[&str; 7] = &[
  "botId",
  "accountId",
  "quoteCurrency",
  "botName",
  "baseCurrency",
  "dealId",
  "strategy",
];

// const BOT_METRIC_NAMES_WITH_CURRENCY: &[&str; 1] = &["profit"];

fn is_safe(value: &str) -> bool {
  value.chars().all(|c| c.is_ascii_alphanumeric())
}

fn write_escaped_tag_value(value: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
  for c in value.chars() {
    match c {
      ' ' => f.write_str("\\ ")?,
      c => f.write_char(c)?,
    }
  }

  Ok(())
}

fn write_measurement<'a, 'b>(
  name: &str,
  tags: &impl TagSet<'a>,
  fields: &impl FieldSet<'b>,
  time: Option<DateTime<Utc>>,
  f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
  f.write_str(name)?;
  for (tag_name, tag_value) in tags.iter() {
    f.write_char(',')?;
    fmt::Display::fmt(tag_name, f)?;
    f.write_char('=')?;
    if is_safe(tag_value) {
      fmt::Display::fmt(tag_value, f)?;
    } else {
      write_escaped_tag_value(tag_value, f)?;
    }
  }

  let mut first = true;
  for (field_name, field_value) in fields.iter() {
    f.write_char(if first {
      first = false;
      ' '
    } else {
      ','
    })?;
    fmt::Display::fmt(field_name, f)?;
    f.write_char('=')?;
    field_value.fmt(f)?;
  }

  if let Some(time) = time {
    f.write_char(' ')?;
    fmt::Display::fmt(&time.timestamp_nanos(), f)?;
  }

  Ok(())
}

fn write_metric<'a, T: TagSet<'a>>(
  metric: &str,
  tags: &T,
  field_name: &'static str,
  field_value: &dyn MetricValue,
  date: DateTime<Utc>,
  f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
  let field_names = [field_name];
  let field_values = [field_value];
  let fields = Fields::new(&field_names, &field_values);
  write_measurement(metric, tags, &fields, Some(date), f)?;
  f.write_char('\n')?;

  Ok(())
}

pub fn write_metrics_for_bot(bot: &Cached<BotData>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
  let cache_time = bot.cache_time();
  let bot_id = bot.id().to_string();
  let account_id = bot.account_id().to_string();
  let quote_currency = bot.currency();
  let bot_name = bot.name();
  let base_order_volume = bot.base_order_volume();
  let safety_order_volume = bot.safety_order_volume();
  let max_safety_orders = bot.max_safety_orders();
  let max_active_deals = bot.max_active_deals();
  let total_budget = bot.total_budget();
  let profits_in_usd = bot.profits_in_usd();
  let open_deals = bot.open_deals();
  let is_enabled = bot.is_enabled();

  let tags = [&*bot_id, &*account_id, quote_currency, &*bot_name];
  let tags = Tags::new(BOT_TAG_NAMES, &tags);

  write_metric(
    "baseOrderVolume",
    &tags,
    "baseOrderVolume",
    &base_order_volume,
    cache_time,
    f,
  )?;
  write_metric(
    "safetyOrderVolume",
    &tags,
    "safetyOrderVolume",
    &safety_order_volume,
    cache_time,
    f,
  )?;
  write_metric(
    "maxSafetyOrders",
    &tags,
    "maxSafetyOrders",
    &max_safety_orders,
    cache_time,
    f,
  )?;
  write_metric(
    "maxActiveDeals",
    &tags,
    "max_active_deals",
    &max_active_deals,
    cache_time,
    f,
  )?;
  write_metric(
    "totalBudget",
    &tags,
    "totalBudget",
    &total_budget,
    cache_time,
    f,
  )?;
  write_metric(
    "profitsInUsd",
    &tags,
    "profitsInUsd",
    &profits_in_usd,
    cache_time,
    f,
  )?;
  write_metric("openDeals", &tags, "openDeals", &open_deals, cache_time, f)?;
  write_metric("enabled", &tags, "enabled", &is_enabled, cache_time, f)?;
  let values: [&dyn MetricValue; 6] = [
    &base_order_volume,
    &safety_order_volume,
    &max_safety_orders,
    &max_active_deals,
    &total_budget,
    &profits_in_usd,
  ];
  let fields = Fields::new(
    &[
      "baseOrderVolume",
      "safetyOrderVolume",
      "maxSafetyOrders",
      "maxActiveDeals",
      "totalBudget",
      "profitsInUsd",
    ],
    &values,
  );
  write_measurement("bot", &tags, &fields, Some(cache_time), f)?;
  f.write_char('\n')?;

  for (base_currency, profits) in bot.profits() {
    let tags = [
      &*bot_id,
      &*account_id,
      quote_currency,
      &*bot_name,
      base_currency,
    ];
    let tags = Tags::new(BOT_TAG_NAMES_WITH_CURRENCY, &tags);

    write_metric("profit", &tags, "profit", &profits, cache_time, f)?;
  }

  for deal in bot.deals() {
    let cache_time = deal.cache_time();
    let deal_id = deal.id().to_string();
    let base_currency = deal.pair().base();
    let strategy = deal.strategy().to_string();
    let tags = [
      &*bot_id,
      &*account_id,
      quote_currency,
      &*bot_name,
      base_currency,
      &*deal_id,
      &*strategy,
    ];
    let tags = Tags::new(DEAL_TAG_NAMES, &tags);

    let finished = deal.is_finished();
    let max_safety_orders = deal.max_safety_orders();
    let completed_safety_orders_count = deal.completed_safety_orders_count();
    let completed_manual_safety_orders_count = deal.completed_manual_safety_orders_count();
    let bought_volume = match deal.bought_volume() {
      None => continue,
      Some(v) => v,
    };
    let reserved_base_coin = deal.reserved_base_coin();
    let actual_profit = deal.actual_profit();
    let actual_usd_profit = deal.actual_usd_profit();
    let values: [&dyn MetricValue; 8] = [
      &finished,
      &max_safety_orders,
      &completed_safety_orders_count,
      &completed_manual_safety_orders_count,
      &bought_volume,
      &reserved_base_coin,
      &actual_profit,
      &actual_usd_profit,
    ];
    let fields = Fields::new(
      &[
        "finished",
        "maxSafetyOrders",
        "completedSafetyOrdersCount",
        "completedManualSafetyOrdersCount",
        "boughtVolume",
        "reservedBaseCoin",
        "actualProfit",
        "actualUsdProfit",
      ],
      &values,
    );

    write_measurement("deal", &tags, &fields, Some(cache_time), f)?;
    f.write_char('\n')?;
  }

  Ok(())
}
