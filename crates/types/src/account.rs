use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct Account {
  pub id: usize,
  pub auto_balance_period: u32,
  pub auto_balance_portfolio_id: u32,
  pub auto_balance_currency_change_limit: u32,
  pub autobalance_enabled: bool,
  pub hedge_mode_available: bool,
  pub hedge_mode_enabled: bool,
  pub is_locked: bool,
  pub smart_trading_supported: bool,
  pub available_for_trading: bool,
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
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub last_auto_balance: DateTime<Utc>,
  /// Sell all to USD/BTC possibility
  pub fast_convert_available: bool,
  pub grid_bots_allowed: bool,
  pub api_key_invalid: bool,
  pub deposit_enabled: bool,
  pub supported_market_types: String,
  pub api_key: String,
  pub name: String,
  pub auto_balance_method: AutoBalanceMethod,
  pub auto_balance_error: String,
  pub customer_id: String,
  pub subaccount_name: String,
  pub lock_reason: String,
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
  pub address: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AutoBalanceMethod {
  #[serde(rename = "time")]
  Time,

  #[serde(rename = "currency_change")]
  CurrencyChange,
}
