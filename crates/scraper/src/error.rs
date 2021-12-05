use color_eyre::Report;
use std::{error::Error, fmt};

pub(crate) trait IntoReport {
  fn into_report(self) -> Report;
}

impl IntoReport for anyhow::Error {
  #[inline]
  fn into_report(self) -> Report {
    Report::new(AnyhowErrorWrap(self))
  }
}

#[repr(transparent)]
struct AnyhowErrorWrap(anyhow::Error);

impl fmt::Debug for AnyhowErrorWrap {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Debug::fmt(&self.0, f)
  }
}

impl fmt::Display for AnyhowErrorWrap {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Display::fmt(&self.0, f)
  }
}

impl Error for AnyhowErrorWrap {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    self.0.source()
  }
}
