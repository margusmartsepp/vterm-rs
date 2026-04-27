//! Aspect: structured tracing span per command.
//!
//! Opens an `info_span!("cmd", kind = …)` around the inner service call. The span is
//! attached to the future via `tracing::Instrument` so it covers every `.await` point
//! inside the dispatcher, not just the synchronous prologue.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use tower::{Layer, Service};
use tracing::Instrument;

use crate::protocol::SkillCommand;

#[derive(Clone, Default)]
pub struct TracingLayer;

impl<S> Layer<S> for TracingLayer {
    type Service = Tracing<S>;
    fn layer(&self, inner: S) -> Self::Service { Tracing { inner } }
}

#[derive(Clone)]
pub struct Tracing<S> { inner: S }

impl<S> Service<SkillCommand> for Tracing<S>
where
    S: Service<SkillCommand> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Response: Send + 'static,
    S::Error: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<S::Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: SkillCommand) -> Self::Future {
        let span = tracing::info_span!("cmd", kind = req.variant_name());
        Box::pin(self.inner.call(req).instrument(span))
    }
}
