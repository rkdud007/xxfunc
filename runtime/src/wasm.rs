use eyre::{eyre, ContextCompat, Result};
use serde_json;
use wasi_common::{pipe::WritePipe, sync::WasiCtxBuilder, WasiCtx};
use wasmtime::{Engine, Instance, Linker, Memory, Module as WasmModule, Store};

type AllocParams = (i64,);
type AllocReturn = i64;
type NotificationParams = (i64, i64);
type NotificationReturn = i64;

pub struct ModuleRunner {
    engine: Engine,
    linker: Linker<WasiCtx>,
}

impl ModuleRunner {
    pub fn new() -> Result<Self> {
        let engine = wasmtime::Engine::default();
        let mut linker = Linker::<WasiCtx>::new(&engine);
        wasi_common::sync::add_to_linker(&mut linker, |s| s).map_err(|err| eyre!(err))?;
        Ok(Self { engine, linker })
    }

    // TODO: make input the exex notification
    pub fn execute(&self, module: WasmModule, input: ()) -> Result<()> {
        let mut module = Module::new(module, &self.engine, &self.linker)?;
        let _ = module.run(Default::default())?;
        Ok(())
    }
}

struct Module {
    memory: Memory,
    instance: Instance,
    store: Store<WasiCtx>,
}

impl Module {
    fn new(module: WasmModule, engine: &Engine, linker: &Linker<WasiCtx>) -> Result<Self> {
        let stdout = std::io::stdout();
        let wasi = WasiCtxBuilder::new().stdout(Box::new(WritePipe::new(stdout))).build();
        let mut store = Store::new(engine, wasi);

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|err| eyre!("failed to instantiate: {err}"))?;

        let memory = instance.get_memory(&mut store, "memory").context("failed to get memory")?;

        Ok(Self { store, instance, memory })
    }

    fn run(&mut self, input: serde_json::Value) -> Result<i64> {
        let serialized_notification = serde_json::to_vec(&input)?;

        // Allocate memory for the notification.
        let data_size = serialized_notification.len() as i64;
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
    fn alloc(&mut self, size: i64) -> Result<i64> {
        let func = self
            .instance
            .get_typed_func::<AllocParams, AllocReturn>(&mut self.store, "alloc")
            .map_err(|err| eyre!("failed to get alloc func: {err}"))?;

        let ptr = func
            .call(&mut self.store, (size,))
            .map_err(|err| eyre::eyre!("failed to call alloc func: {err}"))?;

        Ok(ptr)
    }

    fn process(&mut self, ptr: i64, size: i64) -> Result<i64> {
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
