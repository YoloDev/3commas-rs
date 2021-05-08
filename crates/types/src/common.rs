use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TakeProfitType {
  #[serde(rename = "base")]
  Base,

  #[serde(rename = "total")]
  Total,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfitCurrency {
  #[serde(rename = "quote_currency")]
  Quote,

  #[serde(rename = "base_currency")]
  Base,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StopLossType {
  #[serde(rename = "stop_loss")]
  StopLoss,

  #[serde(rename = "stop_loss_and_disable_bot")]
  StopLossAndDisableBot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolumeType {
  #[serde(rename = "quote_currency")]
  QuoteCurrency,

  #[serde(rename = "base_currency")]
  BaseCurrency,

  #[serde(rename = "percent")]
  Percent,

  #[serde(rename = "xbt")]
  Xbt,
}

impl fmt::Display for VolumeType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let repr = match self {
      VolumeType::QuoteCurrency => "quote_currency",
      VolumeType::BaseCurrency => "base_currency",
      VolumeType::Percent => "percent",
      VolumeType::Xbt => "xbt",
    };

    f.write_str(repr)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Strategy {
  #[serde(rename = "long")]
  Long,

  #[serde(rename = "short")]
  Short,
}

impl fmt::Display for Strategy {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let repr = match self {
      Strategy::Long => "long",
      Strategy::Short => "short",
    };

    f.write_str(repr)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_profit_currency_serde() {
    let json = "\"quote_currency\"";
    let profit_currency: ProfitCurrency =
      serde_json::from_str(json).expect("deserialized successfully");

    assert_eq!(profit_currency, ProfitCurrency::Quote);

    let serialized = serde_json::to_string(&profit_currency).expect("serialized successfully");
    assert_eq!(&*serialized, json);
  }
}
