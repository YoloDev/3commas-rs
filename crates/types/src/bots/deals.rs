use crate::{Pair, ProfitCurrency, StopLossType, Strategy, TakeProfitType, VolumeType};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Debug, Deserialize, Clone)]
pub struct Deal {
  id: usize,
  bot_id: usize,
  max_safety_orders: usize,
  deal_has_error: bool,
  account_id: usize,
  active_safety_orders_count: usize,
  created_at: DateTime<Utc>,
  updated_at: Option<DateTime<Utc>>,
  closed_at: Option<DateTime<Utc>>,
  #[serde(rename = "finished?")]
  is_finished: bool,
  current_active_safety_orders_count: usize,
  /// completed safeties (not including manual)
  completed_safety_orders_count: usize,
  /// completed manual safeties
  completed_manual_safety_orders_count: usize,
  pair: Pair,
  status: DealStatus,
  take_profit: Decimal,
  base_order_volume: Decimal,
  safety_order_volume: Decimal,
  safety_order_step_percentage: Decimal,
  bought_amount: Option<Decimal>,
  bought_volume: Option<Decimal>,
  bought_average_price: Option<Decimal>,
  sold_amount: Option<Decimal>,
  sold_volume: Option<Decimal>,
  sold_average_price: Option<Decimal>,
  take_profit_type: TakeProfitType,
  final_profit: Decimal,
  martingale_coefficient: Decimal,
  martingale_volume_coefficient: Decimal,
  martingale_step_coefficient: Decimal,
  profit_currency: ProfitCurrency,
  stop_loss_type: StopLossType,
  safety_order_volume_type: VolumeType,
  base_order_volume_type: VolumeType,
  from_currency: SmolStr,
  to_currency: SmolStr,
  current_price: Decimal,
  take_profit_price: Option<Decimal>,
  stop_loss_price: Option<Decimal>,
  final_profit_percentage: Decimal,
  actual_profit_percentage: Decimal,
  bot_name: String,
  account_name: String,
  usd_final_profit: Decimal,
  actual_profit: Decimal,
  actual_usd_profit: Decimal,
  failed_message: Option<String>,
  reserved_base_coin: Decimal,
  reserved_second_coin: Decimal,
  trailing_deviation: Decimal,
  /// Highest price met in case of long deal, lowest price otherwise
  trailing_max_price: Option<Decimal>,
  /// Highest price met in TSL in case of long deal, lowest price otherwise
  tsl_max_price: Option<Decimal>,
  strategy: Strategy,
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
