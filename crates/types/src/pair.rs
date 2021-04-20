use std::{borrow::Cow, fmt};

use serde::{de::Error, Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pair {
  quote: SmolStr,
  base: SmolStr,
}

impl fmt::Display for Pair {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}/{}", &*self.base, &*self.quote)
  }
}

impl Pair {
  pub fn new(base: impl AsRef<str>, quote: impl AsRef<str>) -> Self {
    Pair {
      quote: SmolStr::new(quote),
      base: SmolStr::new(base),
    }
  }

  #[inline]
  pub fn quote(&self) -> &str {
    &*self.quote
  }

  #[inline]
  pub fn base(&self) -> &str {
    &*self.base
  }
}

impl Serialize for Pair {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut s = String::with_capacity(self.base.len() + self.quote.len() + 1);
    s.push_str(&self.quote);
    s.push('_');
    s.push_str(&self.base);

    s.serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for Pair {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let pair = <Cow<str> as Deserialize>::deserialize(deserializer)?;
    match pair.split_once('_') {
      Some((quote, base)) => Ok(Pair {
        quote: quote.into(),
        base: base.into(),
      }),
      None => Err(<D::Error as Error>::custom(format!(
        "Expected 3commas pair string on the format of QUOTE_BASE, but got '{}'",
        pair
      ))),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn roundtrip() {
    let json = "\"USDT_ETH\"";
    let pair: Pair = serde_json::from_str(json).expect("parsed successfully");

    assert_eq!(pair.base(), "ETH", "base");
    assert_eq!(pair.quote(), "USDT", "quote");
    assert_eq!(format!("{}", pair), "ETH/USDT");

    let serialized = serde_json::to_string(&pair).expect("serialized successfully");
    assert_eq!(serialized, json);
  }
}
