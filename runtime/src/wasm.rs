use eyre::{eyre, ContextCompat, Result};
use wasi_common::{pipe::WritePipe, sync::WasiCtxBuilder, WasiCtx};
use wasmtime::{Engine, Linker, Module, Store};

type AllocParams = (i64,);
type AllocReturn = i64;
type NotificationParams = (i64, i64);
type NotificationReturn = i64;

pub struct Runtime {
    engine: Engine,
    linker: Linker<WasiCtx>,
}

impl Runtime {
    pub fn new() -> Result<Self> {
        let engine = wasmtime::Engine::default();
        let mut linker = Linker::<WasiCtx>::new(&engine);
        wasi_common::sync::add_to_linker(&mut linker, |s| s).map_err(|err| eyre!(err))?;
        Ok(Self { engine, linker })
    }

    // TODO: make input the exex notification
    pub fn execute(&self, module: Module, input: ()) -> Result<()> {
        let stdout = std::io::stdout();
        let wasi = WasiCtxBuilder::new().stdout(Box::new(WritePipe::new(stdout))).build();
        let mut store = Store::new(&self.engine, wasi);

        let instance = self
            .linker
            .instantiate(&mut store, &module)
            .map_err(|err| eyre!("failed to instantiate: {err}"))?;

        let memory = instance.get_memory(&mut store, "memory").context("failed to get memory")?;

        let alloc_func = instance
            .get_typed_func::<AllocParams, AllocReturn>(&mut store, "alloc")
            .map_err(|err| eyre!("failed to get alloc func: {err}"))?;

        let process_func = instance
            .get_typed_func::<NotificationParams, NotificationReturn>(&mut store, "process")
            .map_err(|err| eyre!("failed to get process func: {err}"))?;

        let serialized_notification = serde_json::to_vec(&serde_json::Value::default())?;

        // Allocate memory for the notification.
        let data_size = serialized_notification.len() as i64;
        let data_ptr = alloc_func
            .call(&mut store, (data_size,))
            .map_err(|err| eyre::eyre!("failed to call alloc func: {err}"))?;

        // Write the notification to the allocated memory.
        memory.write(&mut store, data_ptr as usize, &serialized_notification)?;

        // Call the notification function that will read the allocated memoyry.
        let _ = process_func
            .call(&mut store, (data_ptr, data_size))
            .map_err(|err| eyre::eyre!("failed to call notification func: {err}"))?;

        Ok(())
    }
}
