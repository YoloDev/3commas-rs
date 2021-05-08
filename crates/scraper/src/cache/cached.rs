use chrono::{DateTime, Utc};
use std::{
  borrow,
  cmp::Ordering,
  fmt,
  hash::{Hash, Hasher},
  ops::Deref,
  sync::Arc,
};

pub struct Cached<T: ?Sized> {
  data: Arc<T>,
  fetch_time: DateTime<Utc>,
}

impl<T> Cached<T> {
  pub fn new_at(data: T, fetch_time: DateTime<Utc>) -> Self {
    Self {
      data: Arc::new(data),
      fetch_time,
    }
  }

  pub fn new(data: T) -> Self {
    Self::new_at(data, Utc::now())
  }

  pub fn cache_time(&self) -> DateTime<Utc> {
    self.fetch_time
  }
}

impl<T: ?Sized> borrow::Borrow<T> for Cached<T> {
  fn borrow(&self) -> &T {
    &**self
  }
}

impl<T: ?Sized> AsRef<T> for Cached<T> {
  fn as_ref(&self) -> &T {
    &**self
  }
}

impl<T: ?Sized> Clone for Cached<T> {
  fn clone(&self) -> Self {
    Self {
      data: self.data.clone(),
      fetch_time: self.fetch_time,
    }
  }
}

impl<T: ?Sized + fmt::Display> fmt::Display for Cached<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Display::fmt(&**self, f)
  }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Cached<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Debug::fmt(&**self, f)
  }
}

impl<T: ?Sized> fmt::Pointer for Cached<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Pointer::fmt(&(&**self as *const T), f)
  }
}

impl<T: ?Sized + Hash> Hash for Cached<T> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    (**self).hash(state)
  }
}

impl<T: ?Sized + PartialEq> PartialEq for Cached<T> {
  /// Equality for two `Cached`s.
  ///
  /// Two `Cached`s are equal if their inner values are equal, even if they are
  /// stored in different allocation.
  ///
  /// If `T` also implements `Eq` (implying reflexivity of equality),
  /// two `Cached`s that point to the same allocation are always equal.
  ///
  /// # Examples
  ///
  /// ```
  /// let five = Cached::new(5);
  ///
  /// assert!(five == Cached::new(5));
  /// ```
  #[inline]
  fn eq(&self, other: &Cached<T>) -> bool {
    <T as PartialEq>::eq(&**self, &**other)
  }

  /// Inequality for two `Cached`s.
  ///
  /// Two `Cached`s are unequal if their inner values are unequal.
  ///
  /// If `T` also implements `Eq` (implying reflexivity of equality),
  /// two `Cached`s that point to the same value are never unequal.
  ///
  /// # Examples
  ///
  /// ```
  /// let five = Cached::new(5);
  ///
  /// assert!(five != Cached::new(6));
  /// ```
  #[inline]
  #[allow(clippy::partialeq_ne_impl)]
  fn ne(&self, other: &Cached<T>) -> bool {
    <T as PartialEq>::ne(&**self, &**other)
  }
}

impl<T: ?Sized + PartialOrd> PartialOrd for Cached<T> {
  /// Partial comparison for two `Cached`s.
  ///
  /// The two are compared by calling `partial_cmp()` on their inner values.
  ///
  /// # Examples
  ///
  /// ```
  /// use std::cmp::Ordering;
  ///
  /// let five = Cached::new(5);
  ///
  /// assert_eq!(Some(Ordering::Less), five.partial_cmp(&Cached::new(6)));
  /// ```
  fn partial_cmp(&self, other: &Cached<T>) -> Option<Ordering> {
    (**self).partial_cmp(&**other)
  }

  /// Less-than comparison for two `Cached`s.
  ///
  /// The two are compared by calling `<` on their inner values.
  ///
  /// # Examples
  ///
  /// ```
  /// let five = Cached::new(5);
  ///
  /// assert!(five < Cached::new(6));
  /// ```
  fn lt(&self, other: &Cached<T>) -> bool {
    *(*self) < *(*other)
  }

  /// 'Less than or equal to' comparison for two `Cached`s.
  ///
  /// The two are compared by calling `<=` on their inner values.
  ///
  /// # Examples
  ///
  /// ```
  /// let five = Cached::new(5);
  ///
  /// assert!(five <= Cached::new(5));
  /// ```
  fn le(&self, other: &Cached<T>) -> bool {
    *(*self) <= *(*other)
  }

  /// Greater-than comparison for two `Cached`s.
  ///
  /// The two are compared by calling `>` on their inner values.
  ///
  /// # Examples
  ///
  /// ```
  /// let five = Cached::new(5);
  ///
  /// assert!(five > Cached::new(4));
  /// ```
  fn gt(&self, other: &Cached<T>) -> bool {
    *(*self) > *(*other)
  }

  /// 'Greater than or equal to' comparison for two `Cached`s.
  ///
  /// The two are compared by calling `>=` on their inner values.
  ///
  /// # Examples
  ///
  /// ```
  /// let five = Cached::new(5);
  ///
  /// assert!(five >= Cached::new(5));
  /// ```
  fn ge(&self, other: &Cached<T>) -> bool {
    *(*self) >= *(*other)
  }
}

impl<T: ?Sized + Ord> Ord for Cached<T> {
  /// Comparison for two `Cached`s.
  ///
  /// The two are compared by calling `cmp()` on their inner values.
  ///
  /// # Examples
  ///
  /// ```
  /// use std::cmp::Ordering;
  ///
  /// let five = Cached::new(5);
  ///
  /// assert_eq!(Ordering::Less, five.cmp(&Cached::new(6)));
  /// ```
  fn cmp(&self, other: &Cached<T>) -> Ordering {
    (**self).cmp(&**other)
  }
}
impl<T: ?Sized + Eq> Eq for Cached<T> {}

impl<T: ?Sized> Deref for Cached<T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &T {
    &self.data
  }
}
