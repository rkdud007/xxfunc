use clap::{Parser, Subcommand};
use reqwest::blocking::multipart;
use serde_json::json;
use std::{
    fs::{self},
    path::Path,
    process::Command,
};
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
    Deploy(DeployArgs),
    Start(StartArgs),
}

#[derive(Parser)]
#[clap(about = "Build the project using cargo wasi")]
struct BuildArgs {
    #[clap(long, help = "Build artifacts in release mode, with optimizations")]
    release: bool,
}

#[derive(Parser)]
#[clap(about = "Deploy the function to the xxfunc service")]
struct DeployArgs {
    #[clap(long, help = "URL of the xxfunc service")]
    url: String,

    #[clap(long, help = "Path to the function's WASM file")]
    wasm_path: String,
}

#[derive(Parser)]
#[clap(about = "Start a module on the xxfunc service")]
struct StartArgs {
    #[clap(long, help = "URL of the xxfunc service")]
    url: String,
    #[clap(long, help = "Name of the module to start")]
    module_name: String,
}

fn main() -> eyre::Result<()> {
    let args = Cli::parse();

    match args.command {
        XxfuncCommand::Build(build_args) => build(build_args.release),
        XxfuncCommand::Deploy(deploy_args) => deploy(&deploy_args.url, &deploy_args.wasm_path),
        XxfuncCommand::Start(start_args) => start(&start_args.url, &start_args.module_name),
    }
}

fn build(release: bool) -> eyre::Result<()> {
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
    Ok(())
}

pub fn deploy(url: &str, wasm_file_path: &str) -> eyre::Result<()> {
    let form =
        multipart::Form::new().file("module", wasm_file_path).expect("Failed to create form file");

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&format!("{}/deploy", url))
        .multipart(form)
        .send()
        .expect("Failed to send deploy request");

    if response.status().is_success() {
        println!("Function deployed successfully!");
        println!("Response: {}", response.text()?);
        Ok(())
    } else {
        println!("Deployment failed with status: {}", response.status());
        Err(eyre::eyre!("Deployment failed"))
    }
}

fn start(url: &str, module_name: &str) -> eyre::Result<()> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&format!("{}/start", url))
        .json(&json!({
            "module": module_name
        }))
        .send()
        .expect("Failed to send deploy request");

    if response.status().is_success() {
        println!("Function started successfully!");
        println!("Response: {}", response.text()?);
        Ok(())
    } else {
        println!("Start failed with status: {}", response.status());
        Err(eyre::eyre!("Start failed"))
    }
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
