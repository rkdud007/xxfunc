use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::process::Command;
use toml::Value;

#[derive(Parser)]
#[clap(name = "cargo-xxfunc")]
struct Cli {
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
    let args = Cli::parse();

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
    let project_name = get_project_name().expect("Failed to get project name");
    let wasm_file = target_dir.join(format!("{}.wasm", project_name));
    let dest_dir = Path::new("wasm_output");

    fs::create_dir_all(dest_dir).expect("Failed to create destination directory");
    fs::copy(&wasm_file, dest_dir.join("output.wasm")).expect("Failed to copy WASM file");

    println!("Build completed successfully!");
}

fn get_project_name() -> Result<String, Box<dyn std::error::Error>> {
    let cargo_toml = fs::read_to_string("Cargo.toml")?;
    let cargo_toml: Value = toml::from_str(&cargo_toml)?;

    let package = cargo_toml
        .get("package")
        .ok_or("No [package] section in Cargo.toml")?
        .as_table()
        .ok_or("[package] is not a table")?;

    let name = package
        .get("name")
        .ok_or("No name field in [package]")?
        .as_str()
        .ok_or("name is not a string")?;

    Ok(name.to_string())
}
