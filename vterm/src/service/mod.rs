//! Tower-style command pipeline.
//!
//! ```text
//!   Request ─► CorrelationLayer ─► TimingLayer ─► TracingLayer ─► Dispatcher ─► CommandResult
//!                  (req_id)        (duration_ms)     (span)            (work)
//! ```
//!
//! Each layer is a one-screen `tower::Layer` impl. Adding a new aspect — rate limiting,
//! authentication, retries, audit — is one new layer, no surgery on the dispatcher.

mod correlation;
mod dispatcher;
mod timing;
mod tracing_layer;

use std::convert::Infallible;
use std::sync::Arc;

use tower::ServiceBuilder;

use crate::protocol::{Event, Request, Response};
use crate::session::ConnectionId;
use crate::App;

pub use correlation::CorrelationLayer;
pub use dispatcher::Dispatcher;
pub use timing::TimingLayer;
pub use tracing_layer::TracingLayer;

/// Build the standard pipeline. Returns a `Service` that maps `Request → Response`
/// with `Error = Infallible` (every failure surfaces in the response body, never as
/// a service-level error).
use tokio::sync::mpsc;

pub fn pipeline(
    app: Arc<App>,
    owner: ConnectionId,
    event_tx: mpsc::UnboundedSender<Event>,
) -> impl tower::Service<
    Request,
    Response = Response,
    Error = Infallible,
    Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Response, Infallible>> + Send>,
    >,
> + Clone
       + Send
       + 'static {
    ServiceBuilder::new()
        .layer(CorrelationLayer)
        .layer(TimingLayer)
        .layer(TracingLayer)
        .service(Dispatcher::new(app, owner, event_tx))
}
