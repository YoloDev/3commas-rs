mod deals;
mod errors;
mod middleware;

pub use deals::{Deals, DealsScope};
pub use errors::{ClientError, RequestError};
pub use three_commas_types::{
  Account, AccountId, AutoBalanceMethod, Bot, BotStats, Deal, MarketType, Pair,
};

use middleware::RequestBuilderExt;
use std::result::Result as StdResult;
use std::time::Duration;
use surf::{http::Result, Client, Config, Url};

#[derive(Clone)]
pub struct ThreeCommasClient {
  pub(crate) client: Client,
}

impl ThreeCommasClient {
  pub fn new(api_key: impl AsRef<str>, secret: impl AsRef<str>) -> StdResult<Self, ClientError> {
    let client: Client = Config::new()
      .set_base_url(Url::parse("https://api.3commas.io/public/api/").unwrap())
      .try_into()
      .map_err(ClientError::FailedCreate)?;

    let client = client
      .with(middleware::TracingRequestLoggerMiddlware)
      .with(middleware::ApiKeyMiddleware::new(api_key.as_ref()))
      .with(middleware::SigningMiddleware::new(secret.as_ref()))
      .with(middleware::ErrorHandlerMiddleware)
      .with(middleware::Limit::new(2, Duration::from_secs(1)))
      .with(middleware::Limit::new(30, Duration::from_secs(60)))
      .with(middleware::TracingPipelineLoggerMiddlware);

    Ok(Self { client })
  }

  pub async fn accounts(&self) -> Result<Vec<Account>> {
    let req = self.client.get("ver1/accounts").signed();
    self.client.recv_json(req).await
  }

  pub async fn account(&self, account_id: AccountId) -> Result<Account> {
    let req = self
      .client
      .get(format!("ver1/accounts/{}", account_id))
      .signed();
    self.client.recv_json(req).await
  }

  pub async fn bots(&self) -> Result<Vec<Bot>> {
    let req = self.client.get("ver1/bots").signed();
    self.client.recv_json(req).await
  }

  pub async fn bot_stats(&self, bot: &Bot) -> Result<BotStats> {
    let req = self
      .client
      .get(format!(
        "ver1/bots/stats?account_id={}&bot_id={}",
        bot.account_id(),
        bot.id()
      ))
      .signed();
    self.client.recv_json(req).await
  }

  pub fn deals(&self) -> Deals {
    Deals::new(self.clone())
  }
}
