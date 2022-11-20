#[macro_use]
extern crate log;

use async_std::future::timeout;
use async_std::stream::StreamExt;
use std::error::Error;
use std::future;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Activate subscriber for crates logs.
    tracing_subscriber::FmtSubscriber::builder()
        .compact()
        // Display source code file paths
        .with_file(false)
        // Display source code line numbers
        .with_line_number(true)
        // Display the thread ID an event was recorded on
        .with_thread_ids(false)
        // Don't display the event's target (module path)
        .with_target(false)
        // Read env variable
        .with_env_filter(EnvFilter::from_default_env())
        // Build the subscriber
        .finish()
        .init();

    // launch async-executor's executor (zbus uses that internally)
    let executor = Arc::new(async_executor::Executor::new());
    let executor_clone = executor.clone();
    thread::spawn(move || {
        async_io::block_on(async move {
            while !executor_clone.is_empty() {
                executor_clone.tick().await;
            }

            error!("Executor stopped");
        })
    });

    let (tx, rx) = async_broadcast::broadcast::<usize>(64);
    let rx = rx.deactivate();
    let mut rx2 = rx.activate_cloned();

    // Create the producer task that simulates the socket reading task in zbus.
    executor
        .spawn(async move {
            let mut idx = 1;
            loop {
                timeout(Duration::from_millis(1), future::pending::<()>())
                    .await
                    .unwrap_err();
                if let Err(e) = tx.broadcast(idx).await {
                    error!("Error broadcasting `{}`: {}", idx, e);
                }
                info!("Broadcasted {}", idx);
                idx += 1;
            }
        })
        .detach();

    // Simulate the zbus's ObjectServer task.
    executor
        .spawn(async move {
            while let Some(msg) = rx2.next().await {
                info!("executor task received: {}", msg);
            }
            error!("executor receiver task stopped");
        })
        .detach();

    // And now the problematic bit of receiving in the async-std main task.
    loop {
        debug!("new loop");
        let mut rx = rx.activate_cloned();
        match rx.next().await {
            Some(msg) => info!("main task received: {}", msg),
            None => {
                error!("main rx is closed");

                break;
            }
        }
    }

    Ok(())
}
