mod deals;
mod stats;

use crate::Pair;
use rust_decimal::Decimal;
use serde::Deserialize;

pub use deals::{Deal, DealStatus};
pub use stats::{BotStats, TokenValues};

#[derive(Debug, Deserialize, Clone)]
pub struct Bot {
  pub name: String,
  pub id: usize,
  pub account_id: usize,
  pub is_enabled: bool,
  pub max_safety_orders: usize,
  pub max_active_deals: usize,
  pub base_order_volume: Decimal,
  pub safety_order_volume: Decimal,
  pub safety_order_step_percentage: Decimal,
  pub martingale_volume_coefficient: Decimal,
  pub martingale_step_coefficient: Decimal,
  pub pairs: Vec<Pair>,
}

impl Bot {
  pub fn name(&self) -> &str {
    &*self.name
  }

  pub fn id(&self) -> usize {
    self.id
  }

  pub fn account_id(&self) -> usize {
    self.account_id
  }

  pub fn is_enabled(&self) -> bool {
    self.is_enabled
  }

  pub fn pairs(&self) -> &[Pair] {
    &self.pairs
  }

  pub fn base_order_volume(&self) -> Decimal {
    self.base_order_volume
  }

  pub fn safety_order_volume(&self) -> Decimal {
    self.safety_order_volume
  }

  pub fn max_safety_orders(&self) -> usize {
    self.max_safety_orders
  }

  pub fn max_active_deals(&self) -> usize {
    self.max_active_deals
  }

  pub fn max_budget_per_deal(&self) -> Decimal {
    let mut result = self.base_order_volume;
    let mut next = self.safety_order_volume;
    for _ in 0..self.max_safety_orders {
      result += next;
      next *= self.martingale_volume_coefficient;
    }

    result
  }

  pub fn max_budget(&self) -> Decimal {
    self.max_budget_per_deal() * Decimal::from(self.max_active_deals)
  }
}
