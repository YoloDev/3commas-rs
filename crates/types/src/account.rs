use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{de, Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Deserialize, Clone)]
pub struct Account {
  pub id: AccountId,
  pub auto_balance_period: u32,
  pub auto_balance_portfolio_id: Option<u32>,
  pub auto_balance_currency_change_limit: Option<u32>,
  pub autobalance_enabled: bool,
  pub hedge_mode_available: bool,
  pub hedge_mode_enabled: bool,
  pub is_locked: bool,
  pub smart_trading_supported: bool,
  // pub available_for_trading: bool,
  pub stats_supported: bool,
  pub trading_supported: bool,
  pub market_buy_supported: bool,
  pub market_sell_supported: bool,
  pub conditional_buy_supported: bool,
  pub bots_allowed: bool,
  pub bots_ttp_allowed: bool,
  pub bots_tsl_allowed: bool,
  pub gordon_bots_available: bool,
  pub multi_bots_allowed: bool,
  pub created_at: Option<DateTime<Utc>>,
  pub updated_at: Option<DateTime<Utc>>,
  pub last_auto_balance: Option<DateTime<Utc>>,
  /// Sell all to USD/BTC possibility
  pub fast_convert_available: bool,
  pub grid_bots_allowed: bool,
  pub api_key_invalid: bool,
  pub deposit_enabled: bool,
  pub supported_market_types: Vec<String>,
  pub api_key: Option<String>,
  pub name: String,
  pub auto_balance_method: Option<AutoBalanceMethod>,
  pub auto_balance_error: Option<String>,
  pub customer_id: Option<String>,
  pub subaccount_name: Option<String>,
  pub lock_reason: Option<String>,
  pub btc_amount: Decimal,
  pub usd_amount: Decimal,
  pub day_profit_btc: Decimal,
  pub day_profit_usd: Decimal,
  pub day_profit_btc_percentage: Decimal,
  pub day_profit_usd_percentage: Decimal,
  /// Month period
  pub btc_profit: Decimal,
  /// Month period
  pub usd_profit: Decimal,
  /// Month period
  pub usd_profit_percentage: Decimal,
  /// Month period
  pub btc_profit_percentage: Decimal,
  pub total_btc_profit: Decimal,
  pub total_usd_profit: Decimal,
  pub pretty_display_type: String,
  pub exchange_name: String,
  pub market_code: String,
  pub address: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccountId {
  Summary,

  AccountId(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AutoBalanceMethod {
  #[serde(rename = "time")]
  Time,

  #[serde(rename = "currency_change")]
  CurrencyChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarketType {
  #[serde(rename = "spot")]
  Spot,

  #[serde(rename = "futures")]
  Futures,
}

impl fmt::Display for AccountId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Summary => f.write_str("summary"),
      Self::AccountId(id) => fmt::Display::fmt(id, f),
    }
  }
}

impl Serialize for AccountId {
  #[inline]
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    match self {
      Self::Summary => serializer.serialize_str("summary"),
      Self::AccountId(id) => id.serialize(serializer),
    }
  }
}

struct AccountIdVisitor;
impl<'de> de::Visitor<'de> for AccountIdVisitor {
  type Value = AccountId;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("account id string or number")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    match v {
      "summary" => Ok(AccountId::Summary),
      _ => Err(E::invalid_value(de::Unexpected::Str(v), &self)),
    }
  }

  fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    match u32::try_from(v) {
      Ok(v) => self.visit_u32(v),
      Err(_) => Err(E::invalid_value(de::Unexpected::Unsigned(v), &self)),
    }
  }

  fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(AccountId::AccountId(v))
  }

  fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    match u32::try_from(v) {
      Ok(v) => self.visit_u32(v),
      Err(_) => Err(E::invalid_value(de::Unexpected::Signed(v as i64), &self)),
    }
  }

  fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    match u32::try_from(v) {
      Ok(v) => self.visit_u32(v),
      Err(_) => Err(E::invalid_value(de::Unexpected::Signed(v), &self)),
    }
  }
}

impl<'de> Deserialize<'de> for AccountId {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_any(AccountIdVisitor)
  }
}
