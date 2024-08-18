use std::fs::File;

use eyre::{eyre, ContextCompat, Result};
use serde_json;
use wasmtime::{
    Config, Engine, Instance, Linker, Memory, Module as WasmModule, Store, WasmBacktraceDetails,
};
use wasmtime_wasi::{preview1, DirPerms, FilePerms, OutputFile};

type AllocParams = (u64,);
type AllocReturn = u64;
type NotificationParams = (u64, u64);
type NotificationReturn = u64;

pub struct ModuleRunner {
    engine: Engine,
    linker: Linker<preview1::WasiP1Ctx>,
}

impl ModuleRunner {
    pub fn new() -> Result<Self> {
        // enable async support which requires using the WASI preview1 API
        let mut config = Config::new();
        config.async_support(true);
        config.wasm_backtrace_details(WasmBacktraceDetails::Enable);

        let engine = wasmtime::Engine::new(&config).map_err(|e| eyre!(e))?;
        let mut linker = Linker::<preview1::WasiP1Ctx>::new(&engine);
        preview1::add_to_linker_async(&mut linker, |t| t).map_err(|err| eyre!(err))?;

        Ok(Self { engine, linker })
    }

    // TODO: make input the exex notification
    pub async fn execute(&self, module: WasmModule, input: ()) -> Result<()> {
        let mut module = Module::new(self, module).await?;
        let _ = module.run(Default::default()).await?;
        Ok(())
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }
}

struct Module {
    memory: Memory,
    instance: Instance,
    store: Store<preview1::WasiP1Ctx>,
}

impl Module {
    async fn new(runner: &ModuleRunner, module: WasmModule) -> Result<Self> {
        // let output_file = File::create("wasm_stdout.log")?;
        // let stdout = OutputFile::new(output_file);

        // setup the WASI context, with file access to the reth data directory
        let ctx = wasmtime_wasi::WasiCtxBuilder::new()
            .inherit_stdio()
            .preopened_dir("../random-dir", "./reth", DirPerms::READ, FilePerms::READ)
            .expect("failed to preopened dir")
            .build_p1();

        let mut store = Store::new(&runner.engine, ctx);

        let instance = runner
            .linker
            .instantiate_async(&mut store, &module)
            .await
            .map_err(|err| eyre!("failed to instantiate: {err}"))?;

        let memory = instance.get_memory(&mut store, "memory").context("failed to get memory")?;

        Ok(Self { store, instance, memory })
    }

    async fn run(&mut self, input: serde_json::Value) -> Result<()> {
        let serialized_notification = serde_json::to_vec(&input)?;

        // Allocate memory for the notification.
        let data_size = serialized_notification.len() as u64;
        let ptr = self.alloc(data_size).await?;

        // Write the notification to the allocated memory.
        self.write(ptr as usize, &serialized_notification)?;

        // Call the notification function that will read the allocated memory.
        let _ = self.process(ptr, data_size).await?;

        Ok(())
    }

    // write the buffer to the memory at the given pointer.
    fn write(&mut self, ptr: usize, buffer: &[u8]) -> Result<()> {
        self.memory.write(&mut self.store, ptr, buffer)?;
        Ok(())
    }

    // allocate `size` amount of memory and return the pointer to the allocated memory.
    async fn alloc(&mut self, size: u64) -> Result<u64> {
        let func = self
            .instance
            .get_typed_func::<AllocParams, AllocReturn>(&mut self.store, "alloc")
            .map_err(|err| eyre!("failed to get alloc func: {err}"))?;

        let ptr = func
            .call_async(&mut self.store, (size,))
            .await
            .map_err(|err| eyre::eyre!("failed to call alloc func: {err}"))?;

        Ok(ptr)
    }

    async fn process(&mut self, ptr: u64, size: u64) -> Result<u64> {
        let func = self
            .instance
            .get_typed_func::<NotificationParams, NotificationReturn>(&mut self.store, "process")
            .map_err(|err| eyre!("failed to get process func: {err}"))?;

        let result = func
            .call_async(&mut self.store, (ptr, size))
            .await
            .map_err(|err| eyre::eyre!("failed to call process func: {err}"))?;

        Ok(result)
    }
}
