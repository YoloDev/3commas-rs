use crossbeam::atomic::AtomicCell;
use prometheus::core::{Atomic, GenericGaugeVec, Number};
use rust_decimal::prelude::ToPrimitive;
use std::{
  ops::{self},
  sync::atomic::Ordering,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Decimal(rust_decimal::Decimal);

impl From<rust_decimal::Decimal> for Decimal {
  #[inline]
  fn from(value: rust_decimal::Decimal) -> Self {
    Decimal(value)
  }
}

impl From<Decimal> for rust_decimal::Decimal {
  #[inline]
  fn from(value: Decimal) -> Self {
    value.0
  }
}

impl ops::AddAssign for Decimal {
  #[inline]
  fn add_assign(&mut self, rhs: Self) {
    self.0.add_assign(rhs.0)
  }
}

impl ops::SubAssign for Decimal {
  #[inline]
  fn sub_assign(&mut self, rhs: Self) {
    self.0.sub_assign(rhs.0)
  }
}

pub struct AtomicDecimal(AtomicCell<Decimal>);

impl Number for Decimal {
  fn from_i64(v: i64) -> Self {
    rust_decimal::Decimal::from(v).into()
  }

  fn into_f64(self) -> f64 {
    self.0.to_f64().unwrap()
  }
}

impl Atomic for AtomicDecimal {
  type T = Decimal;

  fn new(val: Self::T) -> Self {
    AtomicDecimal(AtomicCell::new(val))
  }

  fn set(&self, val: Self::T) {
    self.0.store(val)
  }

  fn get(&self) -> Self::T {
    self.0.load()
  }

  fn inc_by(&self, delta: Self::T) {
    loop {
      let current = self.0.load();
      let new = Decimal(current.0 + delta.0);
      if self.0.compare_exchange(current, new).is_ok() {
        break;
      }
    }
  }

  fn dec_by(&self, delta: Self::T) {
    loop {
      let current = self.0.load();
      let new = Decimal(current.0 - delta.0);
      if self.0.compare_exchange(current, new).is_ok() {
        break;
      }
    }
  }
}

pub type DecimalGaugeVec = GenericGaugeVec<AtomicDecimal>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Boolean(bool);

impl From<bool> for Boolean {
  #[inline]
  fn from(value: bool) -> Self {
    Boolean(value)
  }
}

impl From<Boolean> for bool {
  #[inline]
  fn from(value: Boolean) -> Self {
    value.0
  }
}

impl ops::AddAssign for Boolean {
  #[inline]
  fn add_assign(&mut self, rhs: Self) {
    self.0 = self.0 || rhs.0
  }
}

impl ops::SubAssign for Boolean {
  #[inline]
  fn sub_assign(&mut self, rhs: Self) {
    self.0 = self.0 && rhs.0
  }
}

pub struct AtomicBool(std::sync::atomic::AtomicBool);

impl Number for Boolean {
  fn from_i64(v: i64) -> Self {
    if v == 0 {
      Boolean(false)
    } else {
      Boolean(true)
    }
  }

  fn into_f64(self) -> f64 {
    if self.0 {
      1f64
    } else {
      0f64
    }
  }
}

impl Atomic for AtomicBool {
  type T = Boolean;

  fn new(val: Self::T) -> Self {
    AtomicBool(std::sync::atomic::AtomicBool::new(val.into()))
  }

  fn set(&self, val: Self::T) {
    self.0.store(val.0, Ordering::Release)
  }

  fn get(&self) -> Self::T {
    Boolean(self.0.load(Ordering::Acquire))
  }

  fn inc_by(&self, delta: Self::T) {
    loop {
      let current = self.0.load(Ordering::Acquire);
      let new = current && delta.0;
      if self
        .0
        .compare_exchange(current, new, Ordering::SeqCst, Ordering::Release)
        .is_ok()
      {
        break;
      }
    }
  }

  fn dec_by(&self, delta: Self::T) {
    loop {
      let current = self.0.load(Ordering::Acquire);
      let new = current || delta.0;
      if self
        .0
        .compare_exchange(current, new, Ordering::SeqCst, Ordering::Release)
        .is_ok()
      {
        break;
      }
    }
  }
}

pub type BoolGaugeVec = GenericGaugeVec<AtomicBool>;
