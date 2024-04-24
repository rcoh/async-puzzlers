use std::{
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc,
    },
    time::Duration,
};

use tokio::{runtime::Handle, sync::Notify};

pub(crate) trait SyncJob: Send + Sync {
    fn run(&self);
    fn shutdown(&self);
}

struct RealSyncJob {
    shutdown: Arc<AtomicBool>,
    shutdown_complete: Arc<Notify>,
    task_runner: TaskRunner,
}

#[derive(Clone)]
struct TaskRunner {}

impl TaskRunner {
    async fn shutdown(&self) {
        tokio::time::sleep(Duration::from_secs(1)).await
    }
}

pub(crate) fn initialize_real() -> Arc<dyn SyncJob> {
    Arc::new(RealSyncJob {
        shutdown: Default::default(),
        shutdown_complete: Arc::new(Notify::new()),
        task_runner: TaskRunner {},
    })
}

impl SyncJob for RealSyncJob {
    fn run(&self) {
        loop {
            if self.shutdown.load(Relaxed) {
                println!("shutdown marker set. exiting");
                break;
            }
            std::thread::sleep(Duration::from_secs(1));
        }
        self.shutdown_complete.notify_one();
    }

    fn shutdown(&self) {
        println!("shutdown sequence starting");
        self.shutdown.store(true, Relaxed);
        let rt = Handle::current();
        let shutdown_complete = self.shutdown_complete.clone();
        let task_runner = self.task_runner.clone();
        std::thread::spawn(move || {
            println!("preparing to shutdown task runner");
            rt.block_on(task_runner.shutdown());
            println!("task runner shut down; preparing to wait for completion notification");
            rt.block_on(async { shutdown_complete.notified().await });
            println!("complete notification received");
        })
        .join()
        .unwrap();
        println!("shutdown complete");
    }
}
