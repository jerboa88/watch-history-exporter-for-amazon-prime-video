use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal::ctrl_c;
use tokio::sync::Notify;
use tracing::{info, warn};

#[derive(Clone)]
pub struct ShutdownManager {
    shutdown_flag: Arc<AtomicBool>,
    shutdown_notify: Arc<Notify>,
}

impl ShutdownManager {
    pub fn new() -> Self {
        Self {
            shutdown_flag: Arc::new(AtomicBool::new(false)),
            shutdown_notify: Arc::new(Notify::new()),
        }
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown_flag.load(Ordering::SeqCst)
    }

    pub async fn wait_for_shutdown(&self) {
        self.shutdown_notify.notified().await;
    }

    pub fn shutdown(&self) {
        if !self.shutdown_flag.swap(true, Ordering::SeqCst) {
            info!("Shutdown signal received, initiating graceful shutdown...");
            self.shutdown_notify.notify_waiters();
        }
    }

    pub async fn wait_for_signal(&self) -> Result<(), Box<dyn std::error::Error>> {
        ctrl_c().await?;
        info!("Received shutdown signal (Ctrl-C)");
        self.shutdown();
        Ok(())
    }
}

impl Default for ShutdownManager {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn setup_shutdown_handler() -> Result<ShutdownManager, Box<dyn std::error::Error>> {
    let shutdown_manager = ShutdownManager::new();

    // Spawn a task to listen for shutdown signals
    let shutdown_manager_clone = shutdown_manager.clone();
    tokio::spawn(async move {
        if let Err(e) = shutdown_manager_clone.wait_for_signal().await {
            warn!("Error setting up signal handler: {}", e);
        }
    });

    Ok(shutdown_manager)
}