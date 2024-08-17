use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Parser)]
#[clap(bin_name = "cargo")]
enum Cli {
    Xxfunc(XxfuncArgs),
}

#[derive(Parser)]
#[clap(about = "xxfunc build tools")]
struct XxfuncArgs {
    #[clap(subcommand)]
    command: XxfuncCommand,
}

#[derive(Subcommand)]
enum XxfuncCommand {
    Build(BuildArgs),
}

#[derive(Parser)]
#[clap(about = "Build the project using cargo wasi")]
struct BuildArgs {
    #[clap(long, help = "Build artifacts in release mode, with optimizations")]
    release: bool,
}

fn main() {
    let Cli::Xxfunc(args) = Cli::parse();

    match args.command {
        XxfuncCommand::Build(build_args) => build(build_args.release),
    }
}

fn build(release: bool) {
    println!("Building with cargo wasi...");

    let mut args = vec!["wasi", "build"];
    if release {
        args.push("--release");
    }

    let status =
        Command::new("cargo").args(&args).status().expect("Failed to execute cargo wasi build");

    if !status.success() {
        eprintln!("cargo wasi build failed");
        std::process::exit(1);
    }

    // Copy the WASM file to a specific location
    let target_dir = if release {
        Path::new("target/wasm32-wasi/release")
    } else {
        Path::new("target/wasm32-wasi/debug")
    };
    let wasm_file = target_dir.join("your_project_name.wasm");
    let dest_dir = Path::new("wasm_output");

    fs::create_dir_all(dest_dir).expect("Failed to create destination directory");
    fs::copy(&wasm_file, dest_dir.join("output.wasm")).expect("Failed to copy WASM file");

    // Generate any necessary wrapper or interface files
    generate_interface_file(dest_dir);

    println!("Build completed successfully!");
}

fn generate_interface_file(dest_dir: &Path) {
    let interface_content = r#"
    export function alloc(len: i32): i32;
    export function main(ptr: i32, len: i32): i64;
    "#;

    fs::write(dest_dir.join("interface.d.ts"), interface_content)
        .expect("Failed to write interface file");
}
