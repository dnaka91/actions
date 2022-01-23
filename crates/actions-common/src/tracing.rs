//! Set up common tracing logs.

use tracing::Level;
use tracing_subscriber::{filter::Targets, fmt::time, prelude::*};

/// Initialize logging of trace calls to stdout.
///
/// The crate name is usually provided by cargo and can be loaded with the [`env!`] macro.
///
/// # Examples
///
/// ```
/// actions_common::tracing::init(env!("CARGO_CRATE_NAME"));
/// ```
pub fn init(crate_name: &str) {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_timer(time::uptime()),
        )
        .with(
            Targets::new()
                .with_target(crate_name, Level::TRACE)
                .with_default(Level::INFO),
        )
        .init();
}
