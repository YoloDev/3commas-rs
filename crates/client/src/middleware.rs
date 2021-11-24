mod request_limiter;

pub(crate) use request_limiter::Limit;

use crate::errors::RequestError;
use async_trait::async_trait;
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;
use std::sync::atomic::{AtomicUsize, Ordering};
use surf::{
  http::url::Position, middleware::Middleware, Error, Request, RequestBuilder, StatusCode,
};
use tracing::{event, span, Level};
use tracing_futures::Instrument;

type HmacSha256 = Hmac<Sha256>;

pub(crate) struct ApiKeyMiddleware {
  api_key: String,
}

impl ApiKeyMiddleware {
  pub fn new(api_key: &str) -> Self {
    Self {
      api_key: api_key.into(),
    }
  }
}

#[async_trait]
impl Middleware for ApiKeyMiddleware {
  async fn handle(
    &self,
    mut req: surf::Request,
    client: surf::Client,
    next: surf::middleware::Next<'_>,
  ) -> surf::Result<surf::Response> {
    req.set_header("APIKEY", &self.api_key);
    next.run(req, client).await
  }
}

struct RequiresSigning;

pub(crate) struct SigningMiddleware {
  secret: String,
}

impl SigningMiddleware {
  pub fn new(secret: &str) -> Self {
    Self {
      secret: secret.into(),
    }
  }
}

#[async_trait]
impl Middleware for SigningMiddleware {
  async fn handle(
    &self,
    mut req: surf::Request,
    client: surf::Client,
    next: surf::middleware::Next<'_>,
  ) -> surf::Result<surf::Response> {
    if let Some(RequiresSigning) = req.ext() {
      let signature = get_signature(self.secret.as_bytes(), &mut req).await?;
      req.set_header("Signature", signature);
    }
    // req.set_header("APIKEY", &self.api_key);
    next.run(req, client).await
  }
}

async fn get_signature(key: &[u8], req: &mut Request) -> surf::Result<String> {
  // TODO: Handle error?
  let mut mac = HmacSha256::new_from_slice(key).unwrap();
  let url = req.url();
  let rel_url = &url[Position::BeforePath..];
  mac.update(rel_url.as_bytes());

  let body = req.take_body().into_bytes().await?;
  if !body.is_empty() {
    mac.update(&body);
    req.set_body(body);
  }

  let signature = mac.finalize().into_bytes();
  let signature = hex::encode(&signature);
  Ok(signature)
}

pub(crate) struct TracingPipelineLoggerMiddlware;

#[async_trait]
impl Middleware for TracingPipelineLoggerMiddlware {
  async fn handle(
    &self,
    req: Request,
    client: surf::Client,
    next: surf::middleware::Next<'_>,
  ) -> surf::Result<surf::Response> {
    let uri = format!("{}", req.url());
    let method = format!("{}", req.method());
    let span = span!(
      target: "3commas::request",
      Level::INFO,
      "3commas::request",
      %method, %uri,
    );

    next.run(req, client).instrument(span).await
  }
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);
pub(crate) struct TracingRequestLoggerMiddlware;

#[async_trait]
impl Middleware for TracingRequestLoggerMiddlware {
  async fn handle(
    &self,
    req: Request,
    client: surf::Client,
    next: surf::middleware::Next<'_>,
  ) -> surf::Result<surf::Response> {
    let start_time = std::time::Instant::now();
    let uri = format!("{}", req.url());
    let method = format!("{}", req.method());
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    event!(target: "3commas::request", Level::INFO, %method, %uri, id, "sending request");

    let res = next.run(req, client).await?;
    let status = res.status();
    let elapsed = start_time.elapsed();
    if status.is_server_error() {
      event!(target: "3commas::request", Level::ERROR, %method, %uri, id, %status, ?elapsed, "request completed");
    } else if status.is_client_error() {
      event!(target: "3commas::request", Level::WARN, %method, %uri, id, %status, ?elapsed, "request completed");
    } else {
      event!(target: "3commas::request", Level::INFO, %method, %uri, id, %status, ?elapsed, "request completed");
    };

    Ok(res)
  }
}

pub(crate) struct ErrorHandlerMiddleware;

#[async_trait]
impl Middleware for ErrorHandlerMiddleware {
  async fn handle(
    &self,
    req: Request,
    client: surf::Client,
    next: surf::middleware::Next<'_>,
  ) -> surf::Result<surf::Response> {
    let result = next.run(req, client).await?;
    match result.status() {
      StatusCode::Ok => Ok(result),
      StatusCode::TooManyRequests => Err(Error::new(
        StatusCode::TooManyRequests,
        RequestError::RateLimitExceeded,
      )),
      StatusCode::ImATeapot => Err(Error::new(StatusCode::ImATeapot, RequestError::AutoBanned)),
      s => {
        let mut result = result;
        let body = result
          .take_body()
          .into_string()
          .await
          .unwrap_or_else(|_| String::from("Failed to read body"));
        Err(Error::new(s, RequestError::UnexpectedStatusCode(s, body)))
      }
    }
  }
}

pub(crate) trait RequestBuilderExt {
  fn signed(self) -> Request;
}

impl RequestBuilderExt for RequestBuilder {
  fn signed(self) -> Request {
    let mut request = self.build();
    request.set_ext(RequiresSigning);
    request
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use surf::http::{Method, Url};

  const SECRET: &str = "abcd1234";

  #[async_std::test]
  async fn get_signature_test() {
    let secret = SECRET.as_bytes();
    let mut request = Request::builder(
      Method::Get,
      Url::parse("https://api.3commas.io/public/api/ver1/bots").unwrap(),
    )
    .build();

    let signature = get_signature(secret, &mut request).await.unwrap();
    assert_eq!(
      signature,
      "505719450087661e95840d47b8bb2374ad2bb25abdcf0d99d7d410ab9b2f1d3a"
    );
  }
}
