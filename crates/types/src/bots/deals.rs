use crate::{Pair, ProfitCurrency, StopLossType, Strategy, TakeProfitType, VolumeType};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Debug, Deserialize, Clone)]
pub struct Deal {
  pub id: usize,
  pub bot_id: usize,
  pub max_safety_orders: usize,
  pub deal_has_error: bool,
  pub account_id: usize,
  pub active_safety_orders_count: usize,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  pub closed_at: Option<DateTime<Utc>>,
  #[serde(rename = "finished?")]
  pub is_finished: bool,
  pub current_active_safety_orders_count: usize,
  /// completed safeties (not including manual)
  pub completed_safety_orders_count: usize,
  /// completed manual safeties
  pub completed_manual_safety_orders_count: usize,
  pub pair: Pair,
  pub status: DealStatus,
  pub take_profit: Decimal,
  pub base_order_volume: Decimal,
  pub safety_order_volume: Decimal,
  pub safety_order_step_percentage: Decimal,
  pub bought_amount: Option<Decimal>,
  pub bought_volume: Option<Decimal>,
  pub bought_average_price: Option<Decimal>,
  pub sold_amount: Option<Decimal>,
  pub sold_volume: Option<Decimal>,
  pub sold_average_price: Option<Decimal>,
  pub take_profit_type: TakeProfitType,
  pub final_profit: Decimal,
  pub martingale_coefficient: Decimal,
  pub martingale_volume_coefficient: Decimal,
  pub martingale_step_coefficient: Decimal,
  pub profit_currency: ProfitCurrency,
  pub stop_loss_type: StopLossType,
  pub safety_order_volume_type: VolumeType,
  pub base_order_volume_type: VolumeType,
  pub from_currency: SmolStr,
  pub to_currency: SmolStr,
  pub current_price: Decimal,
  pub take_profit_price: Option<Decimal>,
  pub stop_loss_price: Option<Decimal>,
  pub final_profit_percentage: Decimal,
  pub actual_profit_percentage: Decimal,
  pub bot_name: String,
  pub account_name: String,
  pub usd_final_profit: Decimal,
  pub actual_profit: Decimal,
  pub actual_usd_profit: Decimal,
  pub failed_message: Option<String>,
  pub reserved_base_coin: Decimal,
  pub reserved_second_coin: Decimal,
  pub trailing_deviation: Decimal,
  /// Highest price met in case of long deal, lowest price otherwise
  pub trailing_max_price: Option<Decimal>,
  /// Highest price met in TSL in case of long deal, lowest price otherwise
  pub tsl_max_price: Option<Decimal>,
  pub strategy: Strategy,
}

impl Deal {
  pub fn id(&self) -> usize {
    self.id
  }

  pub fn account_id(&self) -> usize {
    self.account_id
  }

  pub fn bot_id(&self) -> usize {
    self.bot_id
  }

  pub fn created_at(&self) -> DateTime<Utc> {
    self.created_at
  }

  pub fn status(&self) -> DealStatus {
    self.status
  }

  pub fn is_finished(&self) -> bool {
    self.is_finished
  }

  pub fn is_active(&self) -> bool {
    !self.is_finished
  }

  pub fn pair(&self) -> &Pair {
    &self.pair
  }

  pub fn strategy(&self) -> Strategy {
    self.strategy
  }

  pub fn max_safety_orders(&self) -> usize {
    self.max_safety_orders
  }

  pub fn completed_safety_orders_count(&self) -> usize {
    self.completed_safety_orders_count
  }

  pub fn completed_manual_safety_orders_count(&self) -> usize {
    self.completed_manual_safety_orders_count
  }

  pub fn bought_volume(&self) -> Option<Decimal> {
    self.bought_volume
  }

  pub fn reserved_base_coin(&self) -> Decimal {
    self.reserved_base_coin
  }

  pub fn actual_profit(&self) -> Decimal {
    self.actual_profit
  }

  pub fn actual_usd_profit(&self) -> Decimal {
    self.actual_usd_profit
  }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DealStatus {
  #[serde(rename = "created")]
  Created,

  #[serde(rename = "base_order_placed")]
  BaseOrderPlaced,

  #[serde(rename = "bought")]
  Bought,

  #[serde(rename = "cancelled")]
  Cancelled,

  #[serde(rename = "completed")]
  Completed,

  #[serde(rename = "failed")]
  Failed,

  #[serde(rename = "panic_sell_pending")]
  PanicSellPending,

  #[serde(rename = "panic_sell_order_placed")]
  PanicSellOrderPlaced,

  #[serde(rename = "panic_sold")]
  PanicSold,

  #[serde(rename = "cancel_pending")]
  CancelPending,

  #[serde(rename = "stop_loss_pending")]
  StopLossPending,

  #[serde(rename = "stop_loss_finished")]
  StopLossFinished,

  #[serde(rename = "stop_loss_order_placed")]
  StopLossOrderPlaced,

  #[serde(rename = "switched")]
  Switched,

  #[serde(rename = "switched_take_profit")]
  SwitchedTakeProfit,

  #[serde(rename = "ttp_activated")]
  TtpActivated,

  #[serde(rename = "ttp_order_placed")]
  TtpOrderPlaced,

  #[serde(rename = "liquidated")]
  Liquidated,

  #[serde(rename = "bought_safety_pending")]
  BoughtSafetyPending,

  #[serde(rename = "bought_take_profit_pending")]
  BoughtTakeProfitPending,

  #[serde(rename = "settled")]
  Settled,
}
