// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

//! Global logger definition and functions

use crate::{counters::STRUCT_LOG_COUNT, info, Event, Metadata};

use console_subscriber;
use once_cell::sync::OnceCell;
use std::sync::Arc;
use tracing::Dispatch;
use tracing_subscriber::prelude::*;

use backtrace::Backtrace;

/// The global `Logger`
static LOGGER: OnceCell<Arc<dyn Logger>> = OnceCell::new();
static DISPATCH: OnceCell<Dispatch> = OnceCell::new();

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

/// Sets the global `Logger` exactly once
pub fn set_global_logger(logger: Arc<dyn Logger>) {
    if LOGGER.set(logger).is_err() {
        eprintln!("Global logger has already been set");
    }

    let _ = tracing::subscriber::set_global_default(
        crate::tracing_adapter::TracingToAptosDataLayer
            .with_subscriber(tracing_subscriber::Registry::default()),
    );

    //tracing_subscriber::registry().with(crate::tracing_adapter::TracingToAptosDataLayer::layer()).init();

    /*
       On smoke tests - Each proccess of the swarm attemtps listenning on the same port -- Need a fix for smoke tests.
       let console_layer = console_subscriber::ConsoleLayer::builder()
           .with_default_env()
           .spawn();

       tracing_subscriber::registry()
           .with(console_layer)
           .init();
    */
}

/// Flush the global `Logger`
pub fn flush() {
    println!("Heya!!");
    info!("Heya!!");
    let bt = Backtrace::new();
    println!("{:?}", bt);
    eprintln!("Serendipity!!");
    if let Some(logger) = LOGGER.get() {
        logger.flush();
    }
}

pub fn set_global_dispatch(dispatch: Dispatch) {
    if DISPATCH.set(dispatch).is_err() {
        eprintln!("Global Dispatcher has already been set");
    }
}

pub fn get_timing_dispatch() -> Option<Dispatch> {
    if let Some(dispatch) = DISPATCH.get() {
        Some(dispatch.clone())
    } else {
        None
    }
}
