use std::{collections::VecDeque, sync::Arc, thread::Thread};

use eyre::Result;
use parking_lot::Mutex;
use std::thread;
use xxfunc_db::ModuleDatabase;

use crate::wasm::ModuleRunner;

struct Task {
    module_id: usize,
    exex_notification: Arc<()>,
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
}

impl Runtime {
    pub fn new() -> Result<Self> {
        let num_workers = thread::available_parallelism()?.get();

        let module_db = ModuleDatabase::open("./db")?;
        let runner = ModuleRunner::new()?;
        let tasks = Mutex::new(VecDeque::new());
        let workers = Mutex::new(Vec::with_capacity(num_workers));
        let inner = Arc::new(Inner { runner, workers, tasks, module_db });

        for _ in 0..num_workers {
            let inner = Arc::clone(&inner);

            thread::spawn(move || {
                loop {
                    // 1. take
                    if let Some(task) = inner.tasks.lock().pop_front() {
                        // perform task until completion

                        // 2. get the appropriate module from the database
                        // 3. instantiate the module
                    } else {
                        break;
                    }
                }

                let handle = thread::current();
                inner.workers.lock().push(handle);
                thread::park();
            });
        }

        Ok(Self { inner })
    }

    fn spawn(&self, module_id: usize, exex_notification: Arc<()>) {}

    fn wake(&self) {
        if let Some(worker) = self.inner.workers.lock().pop() {
            worker.unpark();
        }
    }
}
