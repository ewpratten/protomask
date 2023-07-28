mod http;
#[allow(clippy::module_inception)]
mod metrics;

pub use http::serve_metrics;
pub(crate) use metrics::*;
