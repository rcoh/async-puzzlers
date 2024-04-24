mod dependencies;
use std::sync::Arc;

use dependencies::{initialize_real, SyncJob};
use tokio::signal;

#[derive(Clone)]
struct Application {
    sync_job: Arc<dyn SyncJob>,
}

impl Application {
    fn new() -> Self {
        Application {
            sync_job: initialize_real(),
        }
    }
}

#[tokio::main]
async fn main() {
    let application = Application::new();
    let runnable = application.clone();
    let shutdown = tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();
        application.sync_job.shutdown()
    });
    runnable.sync_job.run();
    shutdown.await.unwrap();
    println!("clean exit")
}
