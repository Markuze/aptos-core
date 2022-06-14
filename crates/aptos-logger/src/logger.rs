// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

//! Global logger definition and functions

use crate::{counters::STRUCT_LOG_COUNT, Event, Metadata};

use once_cell::sync::OnceCell;
use std::sync::Arc;
use tracing_subscriber::Layer;
use tracing_flame::{FlameLayer, FlushGuard};
use std::io::BufWriter;
use std::fs::File;
use tracing_subscriber::{prelude::*, fmt};

/// The global `Logger`
static LOGGER: OnceCell<Arc<dyn Logger>> = OnceCell::new();
static FLAME: OnceCell<FlushGuard<BufWriter<File>>> = OnceCell::new();

/// A trait encapsulating the operations required of a logger.
pub trait Logger: Sync + Send + 'static {
    /// Determines if an event with the specified metadata would be logged
    fn enabled(&self, metadata: &Metadata) -> bool;

    /// Record an event
    fn record(&self, event: &Event);

    /// Flush any buffered events
    fn flush(&self);
}

/// Record a logging event to the global `Logger`
pub(crate) fn dispatch(event: &Event) {
    if let Some(logger) = LOGGER.get() {
        STRUCT_LOG_COUNT.inc();
        logger.record(event)
    }
}

/// Check if the global `Logger` is enabled
pub(crate) fn enabled(metadata: &Metadata) -> bool {
    LOGGER
        .get()
        .map(|logger| logger.enabled(metadata))
        .unwrap_or(false)
}

fn setup_flame_global_subscriber(){
    let fmt_layer = fmt::Layer::default();

    let (flame_layer, flame) = FlameLayer::with_file("/tmp/tracing.folded").unwrap();

    if FLAME.set(flame).is_err() {
        eprintln!("Global logger has already been set");
    }

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(flame_layer)
        .init();
}

/// Sets the global `Logger` exactly once
pub fn set_aptos_global_logger(logger: Arc<dyn Logger>) {

    if LOGGER.set(logger).is_err() {
        eprintln!("Global logger has already been set");
    }

    let _ = tracing::subscriber::set_global_default(
        crate::tracing_adapter::TracingToAptosDataLayer
            .with_subscriber(tracing_subscriber::Registry::default()),
    );
}

pub fn set_global_logger(logger: Arc<dyn Logger>)
{
    setup_flame_global_subscriber();
    //set_aptos_global_logger(loggger); <-- Use this for the Aptos logger
}
/// Flush the global `Logger`
pub fn flush() {
    if let Some(logger) = LOGGER.get() {
        logger.flush();
    }
    if let Some(logger) = FLAME.get() {
        logger.flush();
    }
}