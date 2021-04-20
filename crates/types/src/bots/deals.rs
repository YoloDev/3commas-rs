use crate::Pair;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{
  de::{Error, Expected, Unexpected},
  Deserialize, Serialize,
};
use smol_str::SmolStr;
use std::borrow::Cow;

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
  bought_amount: Decimal,
  bought_volume: Decimal,
  bought_average_price: Decimal,
  sold_amount: Decimal,
  sold_volume: Decimal,
  sold_average_price: Decimal,
  take_profit_type: TakeProfitType,
  final_profit: Decimal,
  martingale_coefficient: Decimal,
  martingale_volume_coefficient: Decimal,
  martingale_step_coefficient: Decimal,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TakeProfitType {
  Base,
  Total,
}

impl Serialize for TakeProfitType {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let str = match self {
      Self::Base => "base",
      Self::Total => "total",
    };

    str.serialize(serializer)
  }
}

struct TakeProfitExpected;
impl Expected for TakeProfitExpected {
  fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("\"base\" or \"total\"")
  }
}

impl<'de> Deserialize<'de> for TakeProfitType {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let status_str = <Cow<str> as Deserialize>::deserialize(deserializer)?;
    match &*status_str {
      "base" => Ok(Self::Base),
      "total" => Ok(Self::Total),
      v => Err(<D::Error as Error>::invalid_value(
        Unexpected::Str(v),
        &TakeProfitExpected,
      )),
    }
  }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DealStatus {
  Created,
  BaseOrderPlaced,
  Bought,
  Cancelled,
  Completed,
  Failed,
  PanicSellPending,
  PanicSellOrderPlaced,
  PanicSold,
  CancelPending,
  StopLossPending,
  StopLossFinished,
  StopLossOrderPlaced,
  Switched,
  SwitchedTakeProfit,
  TtpActivated,
  TtpOrderPlaced,
  Liquidated,
  BoughtSafetyPending,
  BoughtTakeProfitPending,
  Settled,
  Other(SmolStr),
}

impl Serialize for DealStatus {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let str = match self {
      Self::Created => "created",
      Self::BaseOrderPlaced => "base_order_placed",
      Self::Bought => "bought",
      Self::Cancelled => "cancelled",
      Self::Completed => "completed",
      Self::Failed => "failed",
      Self::PanicSellPending => "panic_sell_pending",
      Self::PanicSellOrderPlaced => "panic_sell_order_placed",
      Self::PanicSold => "panic_sold",
      Self::CancelPending => "cancel_pending",
      Self::StopLossPending => "stop_loss_pending",
      Self::StopLossFinished => "stop_loss_finished",
      Self::StopLossOrderPlaced => "stop_loss_order_placed",
      Self::Switched => "switched",
      Self::SwitchedTakeProfit => "switched_take_profit",
      Self::TtpActivated => "ttp_activated",
      Self::TtpOrderPlaced => "ttp_order_placed",
      Self::Liquidated => "liquidated",
      Self::BoughtSafetyPending => "bought_safety_pending",
      Self::BoughtTakeProfitPending => "bought_take_profit_pending",
      Self::Settled => "settled",
      Self::Other(v) => &*v,
    };

    str.serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for DealStatus {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let status_str = <Cow<str> as Deserialize>::deserialize(deserializer)?;
    Ok(match &*status_str {
      "created" => Self::Created,
      "base_order_placed" => Self::BaseOrderPlaced,
      "bought" => Self::Bought,
      "cancelled" => Self::Cancelled,
      "completed" => Self::Completed,
      "failed" => Self::Failed,
      "panic_sell_pending" => Self::PanicSellPending,
      "panic_sell_order_placed" => Self::PanicSellOrderPlaced,
      "panic_sold" => Self::PanicSold,
      "cancel_pending" => Self::CancelPending,
      "stop_loss_pending" => Self::StopLossPending,
      "stop_loss_finished" => Self::StopLossFinished,
      "stop_loss_order_placed" => Self::StopLossOrderPlaced,
      "switched" => Self::Switched,
      "switched_take_profit" => Self::SwitchedTakeProfit,
      "ttp_activated" => Self::TtpActivated,
      "ttp_order_placed" => Self::TtpOrderPlaced,
      "liquidated" => Self::Liquidated,
      "bought_safety_pending" => Self::BoughtSafetyPending,
      "bought_take_profit_pending" => Self::BoughtTakeProfitPending,
      "settled" => Self::Settled,
      v => Self::Other(SmolStr::from(v)),
    })
  }
}
