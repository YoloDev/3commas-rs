use crate::{
  cache::BotData,
  gauges::{AtomicBool, AtomicDecimal},
};
use anyhow::Result;
use prometheus::{
  core::{Atomic, AtomicF64, AtomicU64, GenericGaugeVec},
  Opts, Registry,
};
use rust_decimal::Decimal;
use std::sync::Arc;

pub trait Metric: Copy {
  type DataType: Atomic + 'static;

  fn to_metric_value(value: Self) -> <Self::DataType as Atomic>::T;
}

impl Metric for Decimal {
  type DataType = AtomicDecimal;

  #[inline]
  fn to_metric_value(value: Self) -> <Self::DataType as Atomic>::T {
    value.into()
  }
}

impl Metric for u64 {
  type DataType = AtomicU64;

  #[inline]
  fn to_metric_value(value: Self) -> <Self::DataType as Atomic>::T {
    value
  }
}

impl Metric for usize {
  type DataType = AtomicU64;

  #[inline]
  fn to_metric_value(value: Self) -> <Self::DataType as Atomic>::T {
    value as u64
  }
}

impl Metric for f64 {
  type DataType = AtomicF64;

  #[inline]
  fn to_metric_value(value: Self) -> <Self::DataType as Atomic>::T {
    value
  }
}

impl Metric for bool {
  type DataType = AtomicBool;

  #[inline]
  fn to_metric_value(value: Self) -> <Self::DataType as Atomic>::T {
    value.into()
  }
}

const BOT_LABEL_NAMES: &[&str] = &["bot_id", "account_id", "currency"];
pub struct BotLabels<'a> {
  bot_id: String,
  account_id: String,
  currency: &'a str,
}

impl<'a> BotLabels<'a> {
  pub fn new(bot: &'a BotData) -> Self {
    let bot_id = bot.id().to_string();
    let account_id = bot.account_id().to_string();

    Self {
      bot_id,
      account_id,
      currency: bot.currency(),
    }
  }

  pub fn as_label_values(&self) -> [&str; BOT_LABEL_NAMES.len()] {
    [&*self.bot_id, &*self.account_id, self.currency]
  }
}

#[derive(Clone)]
pub struct BotGauge<T: Metric, const EXTRA_LABELS: usize>(
  Arc<GenericGaugeVec<<T as Metric>::DataType>>,
);

impl<T: Metric> BotGauge<T, 0> {
  pub fn new(name: &'static str, help: &'static str) -> Result<Self> {
    Self::new_with_labels(name, help, &[])
  }

  pub fn set(&self, bot_labels: &BotLabels, value: T) {
    self.set_with_labels(bot_labels, value, &[]);
  }
}

impl<T: Metric, const EXTRA_LABELS: usize> BotGauge<T, EXTRA_LABELS> {
  pub fn new_with_labels(
    name: &'static str,
    help: &'static str,
    extra_label_names: &'static [&'static str; EXTRA_LABELS],
  ) -> Result<Self> {
    let opts = Opts::new(name, help)
      .namespace("three_commas")
      .subsystem("bots");

    let mut labels = Vec::with_capacity(BOT_LABEL_NAMES.len() + EXTRA_LABELS);
    labels.extend(BOT_LABEL_NAMES);
    labels.extend(extra_label_names);
    let gauge = GenericGaugeVec::new(opts, &labels)?;

    Ok(Self(Arc::new(gauge)))
  }

  pub fn register(&self, registry: &Registry) -> Result<()> {
    let inner = &*self.0;
    registry.register(Box::new(inner.clone()))?;
    Ok(())
  }

  pub fn set_with_labels(
    &self,
    bot_labels: &BotLabels,
    value: T,
    extra_labels: &[&str; EXTRA_LABELS],
  ) {
    if EXTRA_LABELS == 0 {
      self
        .0
        .with_label_values(&bot_labels.as_label_values())
        .set(T::to_metric_value(value));
    } else {
      let mut all_label_vals = Vec::with_capacity(BOT_LABEL_NAMES.len() + EXTRA_LABELS);
      all_label_vals.extend(std::array::IntoIter::new(bot_labels.as_label_values()));
      all_label_vals.extend(extra_labels);

      self
        .0
        .with_label_values(&all_label_vals)
        .set(T::to_metric_value(value));
    }
  }
}
