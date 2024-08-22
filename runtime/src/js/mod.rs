use crate::runtime::{JoinHandle, Runtime};
use boa_engine::{Context, JsResult, JsValue, Source};
use eyre::Result;
use futures::channel::oneshot;
use parking_lot::Mutex;
use reth_exex_types::ExExNotification;
use std::{
    collections::VecDeque,
    sync::Arc,
    thread::{self, Thread},
};
use tracing::info;
use xxfunc_db::{ModuleDatabase, ModuleId};

pub struct JsRuntime {
    inner: Arc<Inner>,
}

struct Task {
    module_id: ModuleId,
    exex_notification: Arc<ExExNotification>,
    result_sender: oneshot::Sender<Result<JsValue>>,
}

struct Inner {
    context: Context,
    /// database to fetch modules
    module_db: ModuleDatabase,
    /// Tasks queue
    tasks: Mutex<VecDeque<Task>>,
    /// workers pool
    workers: Mutex<Vec<Thread>>,
    /// to spawn the module execution on bcs to support async
    tokio_runtime: tokio::runtime::Runtime,
}

impl Runtime for JsRuntime {
    type ExecutionResult = JsValue;

    fn new(module_db: ModuleDatabase) -> Result<Self> {
        let num_workers = thread::available_parallelism()?.get();
        let tasks = Mutex::new(VecDeque::new());
        let workers = Mutex::new(Vec::with_capacity(num_workers));
        let tokio_runtime = tokio::runtime::Builder::new_multi_thread().enable_io().build()?;
        let context = Context::default();
        let inner = Arc::new(Inner { context, workers, tasks, module_db, tokio_runtime });

        for _ in 0..num_workers {
            let inner = Arc::clone(&inner);

            thread::spawn(move || loop {
                while let Some(task) = inner.tasks.lock().pop_front() {
                    let bytes = inner.module_db.get(task.module_id).unwrap().unwrap();
                    let module_id = task.module_id;
                    inner.tokio_runtime.block_on(async {
                        info!(%module_id, "Executing module.");

                        let res = inner.context.eval(Source::from_bytes(&bytes));

                        let _ =
                            task.result_sender.send(res.map_err(|err| eyre::eyre!("{:?}", err)));
                    });
                }

                let handle = thread::current();
                inner.workers.lock().push(handle);
                thread::park();
            });
        }

        Ok(Self { inner })
    }

    fn get_db(&self) -> &ModuleDatabase {
        &self.inner.module_db
    }

    fn spawn(
        &self,
        module_id: ModuleId,
        exex_notification: Arc<ExExNotification>,
    ) -> JoinHandle<Result<JsValue>> {
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
