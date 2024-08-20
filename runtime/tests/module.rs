use eyre::Result;
use wasmtime::{Engine, Module};
use xxfunc_runtime::wasm::ModuleRunner;

fn get_test_module(engine: &Engine) -> Module {
    let bytes = include_bytes!("../../examples/wasm_output/output.wasm");
    Module::from_binary(engine, bytes).unwrap()
}

#[tokio::test]
async fn run_module() -> Result<()> {
    let runner = ModuleRunner::new()?;
    let module = get_test_module(runner.engine());
    runner.execute(module, Vec::new()).await?;
    Ok(())
}
