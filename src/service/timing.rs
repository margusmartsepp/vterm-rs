//! Aspect: per-call wall-clock measurement.
//!
//! Wraps any service whose response is a `CommandResult` and stamps `duration_ms`
//! before forwarding the value outward. Generic over the request type — the same
//! layer instruments `SkillCommand → CommandResult` (the inner wire) and would
//! also instrument any future request type that returns `CommandResult`.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

use tower::{Layer, Service};

use crate::protocol::CommandResult;

#[derive(Clone, Default)]
pub struct TimingLayer;

impl<S> Layer<S> for TimingLayer {
    type Service = Timing<S>;
    fn layer(&self, inner: S) -> Self::Service { Timing { inner } }
}

#[derive(Clone)]
pub struct Timing<S> { inner: S }

impl<S, Req> Service<Req> for Timing<S>
where
    S: Service<Req, Response = CommandResult> + Clone + Send + 'static,
    S::Future: Send + 'static,
    Req: Send + 'static,
{
    type Response = CommandResult;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<CommandResult, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        let started = Instant::now();
        let fut = self.inner.call(req);
        Box::pin(async move {
            let mut r = fut.await?;
            r.duration_ms = started.elapsed().as_millis() as u64;
            Ok(r)
        })
    }
}
