use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use tide::{Middleware, Next, Request};
use tracing::{event, span, Level};
use tracing_futures::Instrument;

pub(crate) struct TracingMiddlware;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

impl TracingMiddlware {
  async fn log<'a, State: Clone + Send + Sync + 'static>(
    &'a self,
    req: Request<State>,
    next: Next<'a, State>,
    start_time: std::time::Instant,
    uri: String,
    method: String,
    id: usize,
  ) -> tide::Result {
    event!(Level::INFO, %method, %uri, id, "request received");
    let response = next.run(req).await;
    let status = response.status();
    let elapsed = start_time.elapsed();
    if status.is_server_error() {
      event!(Level::ERROR, %method, %uri, id, %status, ?elapsed, "response sent");
    } else if status.is_client_error() {
      event!(Level::WARN, %method, %uri, id, %status, ?elapsed, "response sent");
    } else {
      event!(Level::INFO, %method, %uri, id, %status, ?elapsed, "response sent");
    };

    Ok(response)
  }
}

#[async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for TracingMiddlware {
  async fn handle(&self, request: Request<State>, next: Next<'_, State>) -> tide::Result {
    let start_time = std::time::Instant::now();
    let uri = format!("{}", request.url());
    let method = format!("{}", request.method());
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);

    let span = span!(
      Level::INFO,
      "server::request",
      %uri,
      %method,
    );

    self
      .log(request, next, start_time, uri, method, id)
      .instrument(span)
      .await
  }
}
