//! Aspect: request → response `req_id` correlation.
//!
//! Outermost layer. Strips `req_id` from the inbound `Request`, hands the
//! `SkillCommand` to the inner stack, and re-attaches `req_id` to the resulting
//! `CommandResult` to form the outbound `Response`.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use tower::{Layer, Service};

use crate::protocol::{CommandResult, Request, Response, SkillCommand};

#[derive(Clone, Default)]
pub struct CorrelationLayer;

impl<S> Layer<S> for CorrelationLayer {
    type Service = Correlation<S>;
    fn layer(&self, inner: S) -> Self::Service { Correlation { inner } }
}

#[derive(Clone)]
pub struct Correlation<S> { inner: S }

impl<S> Service<Request> for Correlation<S>
where
    S: Service<SkillCommand, Response = CommandResult> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let req_id = req.req_id;
        let fut = self.inner.call(req.command);
        Box::pin(async move {
            let result = fut.await?;
            Ok(Response { req_id, result })
        })
    }
}
