use eyre::{eyre, ContextCompat, Result};
use serde_json;
use wasi_common::{pipe::WritePipe, WasiCtx};
use wasmtime::{Config, Engine, Instance, Linker, Memory, Module as WasmModule, Store};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder};

type AllocParams = (u64,);
type AllocReturn = u64;
type NotificationParams = (u64, u64);
type NotificationReturn = u64;

// // Construct the wasm engine with async support enabled.
//   let mut config = Config::new();
//   config.async_support(true);
//   let engine = Engine::new(&config)?;

//   // Add the WASI preview1 API to the linker (will be implemented in terms of
//   // the preview2 API)
//   let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
//   preview1::add_to_linker_async(&mut linker, |t| t)?;

//   // Add capabilities (e.g. filesystem access) to the WASI preview2 context
//   // here. Here only stdio is inherited, but see docs of `WasiCtxBuilder` for
//   // more.
//   let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build_p1();

//   let mut store = Store::new(&engine, wasi_ctx);

//   // Instantiate our 'Hello World' wasm module.
//   // Note: This is a module built against the preview1 WASI API.
//   let module = Module::from_file(&engine, "target/wasm32-wasip1/debug/wasi.wasm")?;
//   let func = linker
//       .module_async(&mut store, "", &module)
//       .await?
//       .get_default(&mut store, "")?
//       .typed::<(), ()>(&store)?;

//   // Invoke the WASI program default function.
//   func.call_async(&mut store, ()).await?;

//   Ok(())

pub struct ModuleRunner {
    engine: Engine,
    linker: Linker<WasiP1Ctx>,
}

impl ModuleRunner {
    pub fn new() -> Result<Self> {
        // enable async support which requires using the WASI preview1 API
        let mut config = Config::new();
        config.async_support(true);

        let engine = wasmtime::Engine::new(&config).map_err(|e| eyre!(e))?;
        let mut linker = Linker::<WasiP1Ctx>::new(&engine);

        preview1::add_to_linker_async(&mut linker, |t| t).map_err(|err| eyre!(err))?;

        Ok(Self { engine, linker })
    }

    // TODO: make input the exex notification
    pub fn execute(&self, module: WasmModule, input: ()) -> Result<()> {
        let mut module = Module::new(self, module)?;
        let _ = module.run(Default::default())?;
        Ok(())
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }
}

struct Module {
    memory: Memory,
    instance: Instance,
    store: Store<WasiP1Ctx>,
}

impl Module {
    fn new(runner: &ModuleRunner, module: WasmModule) -> Result<Self> {
        // setup the WASI context, with file access to the reth data directory
        let ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .preopened_dir("<PATH TO RETH DATADIR>", "./data-dir", DirPerms::READ, FilePerms::READ)
            .expect("failed to preopened dir")
            .build_p1();

        let mut store = Store::new(&runner.engine, ctx);

        let instance = runner
            .linker
            .instantiate(&mut store, &module)
            .map_err(|err| eyre!("failed to instantiate: {err}"))?;

        let memory = instance.get_memory(&mut store, "memory").context("failed to get memory")?;

        Ok(Self { store, instance, memory })
    }

    fn run(&mut self, input: serde_json::Value) -> Result<u64> {
        let serialized_notification = serde_json::to_vec(&input)?;

        // Allocate memory for the notification.
        let data_size = serialized_notification.len() as u64;
        let ptr = self.alloc(data_size)?;

        // Write the notification to the allocated memory.
        self.write(ptr as usize, &serialized_notification)?;

        // Call the notification function that will read the allocated memory.
        let result = self.process(ptr, data_size)?;
        Ok(result)
    }

    // write the buffer to the memory at the given pointer.
    fn write(&mut self, ptr: usize, buffer: &[u8]) -> Result<()> {
        self.memory.write(&mut self.store, ptr, &buffer)?;
        Ok(())
    }

    // allocate `size` amount of memory and return the pointer to the allocated memory.
    fn alloc(&mut self, size: u64) -> Result<u64> {
        let func = self
            .instance
            .get_typed_func::<AllocParams, AllocReturn>(&mut self.store, "alloc")
            .map_err(|err| eyre!("failed to get alloc func: {err}"))?;

        let ptr = func
            .call(&mut self.store, (size,))
            .map_err(|err| eyre::eyre!("failed to call alloc func: {err}"))?;

        Ok(ptr)
    }

    fn process(&mut self, ptr: u64, size: u64) -> Result<u64> {
        let func = self
            .instance
            .get_typed_func::<NotificationParams, NotificationReturn>(&mut self.store, "process")
            .map_err(|err| eyre!("failed to get process func: {err}"))?;

        let result = func
            .call(&mut self.store, (ptr, size))
            .map_err(|err| eyre::eyre!("failed to call process func: {err}"))?;

        Ok(result)
    }
}
