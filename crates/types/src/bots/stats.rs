use rust_decimal::Decimal;
use serde::Deserialize;
use smol_str::SmolStr;
use std::{
  collections::{hash_map, HashMap},
  iter::FusedIterator,
};

#[derive(Debug, Deserialize, Clone)]
pub struct BotStats {
  overall_stats: HashMap<SmolStr, Decimal>,
  today_stats: HashMap<SmolStr, Decimal>,
  profits_in_usd: ProfitsInUsd,
}

impl BotStats {
  pub fn overall(&self) -> TokenValues {
    TokenValues {
      values: &self.overall_stats,
    }
  }

  pub fn today(&self) -> TokenValues {
    TokenValues {
      values: &self.today_stats,
    }
  }

  pub fn overall_usd_profit(&self) -> Decimal {
    self.profits_in_usd.overall_usd_profit
  }

  pub fn today_usd_profit(&self) -> Decimal {
    self.profits_in_usd.today_usd_profit
  }

  pub fn active_deals_usd_profit(&self) -> Decimal {
    self.profits_in_usd.active_deals_usd_profit
  }
}

#[derive(Clone)]
pub struct TokenValues<'a> {
  values: &'a HashMap<SmolStr, Decimal>,
}

impl<'a> TokenValues<'a> {
  pub fn get(&self, token: &str) -> Option<Decimal> {
    self.values.get(token).copied()
  }

  pub fn tokens(&self) -> impl Iterator<Item = &'a str> {
    self.values.keys().map(|k| &**k)
  }

  pub fn iter(&self) -> TokenValuesIter<'a> {
    TokenValuesIter {
      iter: self.values.iter(),
    }
  }
}

impl<'a> IntoIterator for TokenValues<'a> {
  type Item = (&'a str, Decimal);
  type IntoIter = TokenValuesIter<'a>;

  fn into_iter(self) -> Self::IntoIter {
    TokenValuesIter {
      iter: self.values.iter(),
    }
  }
}

#[derive(Clone)]
pub struct TokenValuesIter<'a> {
  iter: hash_map::Iter<'a, SmolStr, Decimal>,
}

impl<'a> Iterator for TokenValuesIter<'a> {
  type Item = (&'a str, Decimal);

  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next().map(|(k, v)| (&**k, *v))
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.iter.size_hint()
  }
}

impl<'a> ExactSizeIterator for TokenValuesIter<'a> {
  #[inline]
  fn len(&self) -> usize {
    self.iter.len()
  }
}

impl<'a> FusedIterator for TokenValuesIter<'a> {}

#[derive(Debug, Deserialize, Clone)]
struct ProfitsInUsd {
  overall_usd_profit: Decimal,
  today_usd_profit: Decimal,
  active_deals_usd_profit: Decimal,
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::str::FromStr;

  #[test]
  fn deserialize() {
    let json = r###"
    {
      "overall_stats": {
        "USDT": "79.77234945",
        "BTC": "0.00074279"
      },
      "today_stats": {
        "BTC": "0.00006813",
        "USDT": "10.96327719"
      },
      "profits_in_usd": {
        "overall_usd_profit": 123.49,
        "today_usd_profit": 14.97,
        "active_deals_usd_profit": -8.52
      }
    }
    "###;

    let stats: BotStats = serde_json::from_str(json).expect("deserialize successfully");
    assert_eq!(stats.overall_stats.len(), 2);
    assert_eq!(
      stats.overall_stats.get("USDT"),
      Some(&Decimal::from_str("79.77234945").unwrap())
    );
    assert_eq!(
      stats.overall_stats.get("BTC"),
      Some(&Decimal::from_str("0.00074279").unwrap())
    );
    assert_eq!(
      stats.today_stats.get("USDT"),
      Some(&Decimal::from_str("10.96327719").unwrap())
    );
    assert_eq!(
      stats.today_stats.get("BTC"),
      Some(&Decimal::from_str("0.00006813").unwrap())
    );
    assert_eq!(
      stats.profits_in_usd.overall_usd_profit,
      Decimal::from_str("123.49").unwrap()
    );
    assert_eq!(
      stats.profits_in_usd.today_usd_profit,
      Decimal::from_str("14.97").unwrap()
    );
    assert_eq!(
      stats.profits_in_usd.active_deals_usd_profit,
      Decimal::from_str("-8.52").unwrap()
    );
  }
}
