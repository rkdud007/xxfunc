use eyre::Result;
use wasmtime::{Engine, Module};
use xxfunc_runtime::wasm::ModuleRunner;

fn get_test_minimal_module(engine: &Engine) -> Module {
    let bytes = include_bytes!("../../examples/minimal/wasm_output/output.wasm");
    Module::from_binary(engine, bytes).unwrap()
}

fn get_test_async_module(engine: &Engine) -> Module {
    let bytes = include_bytes!("../../examples/async/wasm_output/output.wasm");
    Module::from_binary(engine, bytes).unwrap()
}

#[tokio::test]
async fn run_module() -> Result<()> {
    let runner = ModuleRunner::new()?;
    let module = get_test_minimal_module(runner.engine());
    runner.execute(module, Vec::new()).await?;
    Ok(())
}

#[tokio::test]
async fn run_async_module() -> Result<()> {
    let runner = ModuleRunner::new()?;
    let module = get_test_async_module(runner.engine());
    runner.execute(module, Vec::new()).await?;
    Ok(())
}
