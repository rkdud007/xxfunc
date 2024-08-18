use std::{collections::VecDeque, sync::Arc, thread::Thread};

use eyre::Result;
use futures::channel::oneshot;
use parking_lot::Mutex;
use std::thread;
use tracing::info;
use wasmtime::Module;
use xxfunc_db::{ModuleDatabase, ModuleId};

use crate::wasm::ModuleRunner;

#[derive(Debug)]
pub struct JoinHandle<T>(oneshot::Receiver<T>);

impl<T> std::future::Future for JoinHandle<T> {
    type Output = Result<T, oneshot::Canceled>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        std::pin::Pin::new(&mut self.0).poll(cx)
    }
}

struct Task {
    module_id: ModuleId,
    exex_notification: Arc<()>,
    result_sender: oneshot::Sender<Result<()>>,
}

pub struct Runtime {
    inner: Arc<Inner>,
}

struct Inner {
    /// runner for executing user modules
    runner: ModuleRunner,
    /// database to fetch modules
    module_db: ModuleDatabase,
    /// Tasks queue
    tasks: Mutex<VecDeque<Task>>,
    /// workers pool
    workers: Mutex<Vec<Thread>>,
    /// to spawn the module execution on bcs to support async
    tokio_runtime: tokio::runtime::Runtime,
}

impl Runtime {
    pub fn new(module_db: ModuleDatabase) -> Result<Self> {
        let num_workers = thread::available_parallelism()?.get();
        let runner = ModuleRunner::new()?;
        let tasks = Mutex::new(VecDeque::new());
        let workers = Mutex::new(Vec::with_capacity(num_workers));
        let tokio_runtime = tokio::runtime::Builder::new_multi_thread().enable_io().build()?;

        let inner = Arc::new(Inner { runner, workers, tasks, module_db, tokio_runtime });

        for _ in 0..num_workers {
            let inner = Arc::clone(&inner);

            thread::spawn(move || {
                loop {
                    while let Some(task) = inner.tasks.lock().pop_front() {
                        // get module from db
                        let bytes = inner.module_db.get(task.module_id).unwrap().unwrap();

                        // deserialize module
                        let engine = inner.runner.engine();
                        let module = unsafe { Module::deserialize(engine, bytes).unwrap() };

                        // execute the module on the tokio runtime because it's async
                        let func = inner.runner.execute(module, ());

                        let module_id = task.module_id;
                        inner.tokio_runtime.block_on(async move {
                            info!(%module_id, "Executing module.");
                            let res = func.await;
                            let _ = task.result_sender.send(res);
                        });
                    }

                    // park thread if no tasks
                    let handle = thread::current();
                    inner.workers.lock().push(handle);
                    thread::park();
                }
            });
        }

        Ok(Self { inner })
    }

    pub async fn spawn(
        &self,
        module_id: ModuleId,
        exex_notification: Arc<()>,
    ) -> JoinHandle<Result<()>> {
        let (result_sender, rx) = oneshot::channel();

        // create task
        let task = Task { module_id, exex_notification, result_sender };
        self.inner.tasks.lock().push_back(task);

        // wake up available worker
        self.wake();

        JoinHandle(rx)
    }

    fn wake(&self) {
        if let Some(worker) = self.inner.workers.lock().pop() {
            worker.unpark();
        }
    }
}

#[cfg(test)]
mod tests {
    use xxfunc_db::ModuleState;

    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_runtime() -> Result<()> {
        // Create a test database and insert a test module
        let db = ModuleDatabase::create_test_db()?;

        db.set_state("test_module", ModuleState::Started)?;

        // Create a new runtime
        let runtime = Runtime::new(db.clone())?;

        // Get the test module ID
        let module_id = db.get_modules_by_state(ModuleState::Started)?[0];

        // Create a mock ExEx notification
        let exex_notification = Arc::new(());

        // Spawn a task on the runtime
        let handle = runtime.spawn(module_id, exex_notification).await;

        // Wait for the task to complete
        let result = handle.await?;

        // Assert that the task completed successfully
        assert!(result.is_ok());

        Ok(())
    }
}
