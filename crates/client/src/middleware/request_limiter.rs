use async_std::{sync::Mutex, task};
use async_trait::async_trait;
use std::time::{Duration, Instant};
use surf::middleware::Middleware;

struct Window {
  start: Instant,
  remaining: usize,
}

impl Window {
  fn new(max: usize) -> Self {
    Self {
      start: Instant::now(),
      remaining: max,
    }
  }
}

pub(crate) struct Limit {
  max: usize,
  window_size: Duration,
  window: Mutex<Window>,
}

impl Limit {
  pub(crate) fn new(max: usize, window_size: Duration) -> Self {
    Self {
      max,
      window_size,
      window: Mutex::new(Window::new(max)),
    }
  }

  async fn wait_for(&self) {
    let wait_duration = {
      let mut guard = self.window.lock().await;
      let elapsed = guard.start.elapsed();
      if elapsed >= self.window_size {
        *guard = Window::new(self.max);
        None
      } else if guard.remaining == 0 {
        debug_assert!(self.window_size > elapsed);
        Some(self.window_size - elapsed)
      } else {
        guard.remaining -= 1;
        None
      }
    };

    if let Some(wait_duration) = wait_duration {
      // println!("Waiting {}ms", wait_duration.as_millis());
      task::sleep(wait_duration).await;
    }
  }
}

#[async_trait]
impl Middleware for Limit {
  async fn handle(
    &self,
    req: surf::Request,
    client: surf::Client,
    next: surf::middleware::Next<'_>,
  ) -> surf::Result<surf::Response> {
    self.wait_for().await;
    next.run(req, client).await
  }
}
