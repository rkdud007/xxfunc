use eyre::Result;
use wasmtime::{Engine, Module};
use xxfunc_runtime::wasm::ModuleRunner;

fn get_test_module(engine: &Engine) -> Module {
    let bytes = include_bytes!("../../example/wasm_output/output.wasm");
    Module::from_binary(engine, bytes).unwrap()
}

#[test]
fn run_module() -> Result<()> {
    let runner = ModuleRunner::new()?;
    let module = get_test_module(runner.engine());
    runner.execute(module, ())?;
    Ok(())
}
