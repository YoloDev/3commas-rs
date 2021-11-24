use crate::{middleware::RequestBuilderExt, ThreeCommasClient};
use futures::{future::BoxFuture, stream::FusedStream, FutureExt, Stream};
use smol_str::SmolStr;
use std::{
  fmt,
  pin::Pin,
  task::{Context, Poll},
  usize, vec,
};
use surf::http::Result;
use three_commas_types::Deal;
use tracing::{event, Level};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DealsScope {
  /// active deals
  Active,
  /// finished deals
  Finished,
  /// successfully completed
  Completed,
  /// cancelled deals
  Cancelled,
  /// failed deals
  Failed,
}

impl DealsScope {
  pub fn as_str(&self) -> &'static str {
    match self {
      DealsScope::Active => "active",
      DealsScope::Finished => "finished",
      DealsScope::Completed => "completed",
      DealsScope::Cancelled => "cancelled",
      DealsScope::Failed => "failed",
    }
  }
}

impl fmt::Display for DealsScope {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_str())
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DealsOrder {
  CreatedAt(DealsOrderDirection),
  UpdatedAt(DealsOrderDirection),
  ClosedAt(DealsOrderDirection),
  Profit(DealsOrderDirection),
  ProfitPercentage(DealsOrderDirection),
}

impl DealsOrder {
  fn to_string_parts(self) -> (&'static str, &'static str) {
    match self {
      DealsOrder::CreatedAt(order) => ("created_at", order.as_str()),
      DealsOrder::UpdatedAt(order) => ("updated_at", order.as_str()),
      DealsOrder::ClosedAt(order) => ("closed_at", order.as_str()),
      DealsOrder::Profit(order) => ("profit", order.as_str()),
      DealsOrder::ProfitPercentage(order) => ("profit_percentage", order.as_str()),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DealsOrderDirection {
  Asc,
  Desc,
}

impl DealsOrderDirection {
  pub fn as_str(&self) -> &'static str {
    match self {
      DealsOrderDirection::Asc => "asc",
      DealsOrderDirection::Desc => "desc",
    }
  }
}

impl fmt::Display for DealsOrderDirection {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_str())
  }
}

type FetchResponse = Vec<Deal>;

struct Inner {
  client: ThreeCommasClient,
  /// Limit records. Max: 1_000
  limit: usize,
  /// Offset records
  offset: Option<usize>,
  /// Account to show bots on. Return all if not specified. Gather this from GET /ver1/accounts
  account_id: Option<usize>,
  /// Bot show deals on. Return all if not specified
  bot_id: Option<usize>,
  /// Limit deals to those in a certain state
  scope: Option<DealsScope>,
  /// Order the results
  order: Option<DealsOrder>,
  /// Base currency
  base: Option<SmolStr>,
  /// Quote currency
  quote: Option<SmolStr>,
}

impl Inner {
  /// Returns a list of deals, and the limit (number of deals requested).
  /// If deals.len() == limit - there might be more deals to load.
  fn request(&self, offset: usize) -> Result<(BoxFuture<'static, Result<FetchResponse>>, usize)> {
    let mut params = form_urlencoded::Serializer::new(String::new());

    let limit_num = self.limit;
    let limit = limit_num.to_string();
    let offset = (self.offset.unwrap_or_default() + offset).to_string();
    let account_id = self.account_id.map(|v| v.to_string());
    let bot_id = self.bot_id.map(|v| v.to_string());
    let scope = self.scope.map(|v| v.as_str());
    let order = self.order.map(|v| v.to_string_parts());
    let base = self.base.as_deref();
    let quote = self.quote.as_deref();

    params.append_pair("limit", &*limit);
    params.append_pair("offset", &*offset);

    if let Some(account_id) = &account_id {
      params.append_pair("account_id", &**account_id);
    }

    if let Some(bot_id) = &bot_id {
      params.append_pair("bot_id", &**bot_id);
    }

    if let Some(scope) = &scope {
      params.append_pair("scope", *scope);
    }

    if let Some((order, direction)) = &order {
      params.append_pair("order", *order);
      params.append_pair("order_direction", *direction);
    }

    if let Some(base) = &base {
      params.append_pair("base", *base);
    }

    if let Some(quote) = &quote {
      params.append_pair("quote", *quote);
    }

    let client = self.client.client.clone();
    let mut url = String::from("ver1/deals?");
    url += &params.finish();
    let deals_fut = async move {
      let req = client.get(url).signed();
      let deals: Result<FetchResponse> = client.recv_json(req).await;
      deals
    };
    Ok((Box::pin(deals_fut), limit_num))
  }
}

enum State {
  Init,
  Fetch(usize),
  Fetching {
    fut: BoxFuture<'static, Result<FetchResponse>>,
    limit: usize,
    offset: usize,
  },
  Yielding {
    iter: vec::IntoIter<Deal>,
    next_offset: Option<usize>,
  },
  Done,
}

pub struct Deals {
  inner: Inner,
  state: State,
}

impl Deals {
  pub(crate) fn new(client: ThreeCommasClient) -> Self {
    Self {
      inner: Inner {
        client,
        limit: 50,
        offset: None,
        account_id: None,
        bot_id: None,
        scope: None,
        order: None,
        base: None,
        quote: None,
      },
      state: State::Init,
    }
  }

  pub fn limit(mut self, limit: usize) -> Self {
    assert!(!(limit > 1000), "limit cannot be greater than 1000");
    assert!(!(limit == 0), "limit cannot be 0");

    self.state = State::Init;
    self.inner.limit = limit;
    self
  }

  pub fn offset(mut self, offset: Option<usize>) -> Self {
    self.state = State::Init;
    self.inner.offset = offset;
    self
  }

  pub fn account_id(mut self, account_id: Option<usize>) -> Self {
    self.state = State::Init;
    self.inner.account_id = account_id;
    self
  }

  pub fn bot_id(mut self, bot_id: Option<usize>) -> Self {
    self.state = State::Init;
    self.inner.bot_id = bot_id;
    self
  }

  pub fn scope(mut self, scope: Option<DealsScope>) -> Self {
    self.state = State::Init;
    self.inner.scope = scope;
    self
  }

  pub fn order(mut self, order: Option<DealsOrder>) -> Self {
    self.state = State::Init;
    self.inner.order = order;
    self
  }

  pub fn base(mut self, base: Option<impl AsRef<str>>) -> Self {
    self.state = State::Init;
    self.inner.base = base.map(|v| SmolStr::from(v.as_ref()));
    self
  }

  pub fn quote(mut self, quote: Option<impl AsRef<str>>) -> Self {
    self.state = State::Init;
    self.inner.quote = quote.map(|v| SmolStr::from(v.as_ref()));
    self
  }
}

impl Stream for Deals {
  type Item = Result<Deal>;

  fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    let this = self.get_mut();
    loop {
      let inner = &this.inner;
      let next = match &mut this.state {
        State::Init => Ok(State::Fetch(0)),
        State::Fetch(offset) => match inner.request(*offset) {
          Ok((fut, limit)) => Ok(State::Fetching {
            fut,
            limit,
            offset: *offset,
          }),
          Err(error) => Err(error),
        },

        State::Fetching { fut, limit, offset } => match fut.poll_unpin(cx) {
          Poll::Ready(Ok(deals)) => {
            let len = deals.len();
            let has_more = len == *limit;
            let iter = deals.into_iter();
            let next_offset = has_more.then(|| len + *offset);
            event!(
              target: "3commas::client::deals",
              Level::DEBUG,
              deals_len = %len,
              offset = %*offset,
              next_offset = ?next_offset,
              "Got {} deals when requesting {}, next offset = {:?}",
              len,
              *limit,
              next_offset
            );

            Ok(State::Yielding { iter, next_offset })
          }
          Poll::Ready(Err(error)) => Err(error),
          Poll::Pending => return Poll::Pending,
        },

        State::Yielding { iter, next_offset } => {
          if let Some(next) = iter.next() {
            return Poll::Ready(Some(Ok(next)));
          }

          match next_offset {
            None => Ok(State::Done),
            Some(offset) => Ok(State::Fetch(*offset)),
          }
        }

        State::Done => return Poll::Ready(None),
      };

      match next {
        Err(error) => {
          this.state = State::Done;
          return Poll::Ready(Some(Err(error)));
        }
        Ok(state) => this.state = state,
      }
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    match &self.state {
      State::Yielding {
        iter,
        next_offset: _,
      } => (iter.len(), None),
      State::Done => (0, Some(0)),
      _ => (0, None),
    }
  }
}

impl FusedStream for Deals {
  fn is_terminated(&self) -> bool {
    matches!(&self.state, State::Done)
  }
}
