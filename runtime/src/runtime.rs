use std::{
    sync::Arc,
    thread::{JoinHandle, Thread},
};

use eyre::{eyre, ContextCompat, Result};
use parking_lot::Mutex;
use std::thread;
use wasi_common::{pipe::WritePipe, sync::WasiCtxBuilder, WasiCtx};
use wasmtime::{Engine, Linker, Module, Store};

use crate::wasm::ModuleRunner;

pub struct Runtime {
    inner: Arc<Inner>,
}

struct Inner {
    /// runner for executing user modules
    runner: ModuleRunner,
    /// workers pool
    workers: Mutex<Vec<Thread>>,
}

impl Runtime {
    pub fn new() -> Result<Self> {
        let num_workers = thread::available_parallelism()?.get();

        let runner = ModuleRunner::new()?;
        let workers = Mutex::new(Vec::with_capacity(num_workers));
        let inner = Arc::new(Inner { runner, workers });

        for _ in 0..num_workers {
            let inner = Arc::clone(&inner);

            thread::spawn(move || {
                loop {
                    // TODO: worker main loop

                    // 1. take
                    // 2. get the appropriate module from the database
                    // 3. instantiate the module
                    // 4. get the exex nofitication to prepare as module input

                    break;
                }

                let handle = thread::current();
                inner.workers.lock().push(handle);
                thread::park();
            });
        }

        Ok(Self { inner })
    }

    fn spawn(&self, module_id: usize) {}

    fn wake(&self) {
        if let Some(worker) = self.inner.workers.lock().pop() {
            worker.unpark();
        }
    }
}
