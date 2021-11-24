use surf::{Client, Config, StatusCode};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RequestError {
  #[error("Rate limit exceeded")]
  RateLimitExceeded,

  #[error("Auto banned for exceeding rate limit")]
  AutoBanned,

  #[error("Unexpected status code {0}. Response body: {1}")]
  UnexpectedStatusCode(StatusCode, String),
}

#[derive(Debug, Error)]
pub enum ClientError {
  #[error("Failed to create client")]
  FailedCreate(<Client as TryFrom<Config>>::Error),
}
