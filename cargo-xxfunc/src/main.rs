use args::{build, deploy, start, BuildArgs, DeployArgs, StartArgs};
use clap::{Parser, Subcommand};

mod args;

#[derive(Parser)]
#[clap(name = "cargo-xxfunc")]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Build(BuildArgs),
    Deploy(DeployArgs),
    Start(StartArgs),
}

fn main() -> eyre::Result<()> {
    let args = Cli::parse();

    match args.command {
        Command::Build(build_args) => build(build_args.release),
        Command::Deploy(deploy_args) => deploy(&deploy_args.url, &deploy_args.wasm_path),
        Command::Start(start_args) => start(&start_args.url, &start_args.module_name),
    }
}
