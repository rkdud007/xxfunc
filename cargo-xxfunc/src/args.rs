use clap::Parser;
use eyre::eyre;
use reqwest::blocking::multipart;
use serde_json::json;
use std::{fs, path::Path, process::Command};
use toml::Value;

#[derive(Parser)]
#[clap(about = "Build the project using cargo wasi")]
pub struct BuildArgs {
    #[clap(long, short, help = "Build artifacts in release mode")]
    pub release: bool,
}

#[derive(Parser)]
#[clap(about = "Deploy the module to the xxfunc service")]
pub struct DeployArgs {
    #[clap(long, help = "URL of the xxfunc service")]
    pub url: String,

    #[clap(long, help = "Path to the module's WASM file")]
    pub wasm_path: String,
}

#[derive(Parser)]
#[clap(about = "Start a module on the xxfunc service")]
pub struct StartArgs {
    #[clap(long, help = "URL of the xxfunc service")]
    pub url: String,

    #[clap(long, help = "Name of the module to start")]
    pub module_name: String,
}

#[derive(Parser)]
#[clap(about = "Stop a module on the xxfunc service")]
pub struct StopArgs {
    #[clap(long, help = "URL of the xxfunc service")]
    pub url: String,

    #[clap(long, help = "Name of the module to stop")]
    pub module_name: String,
}

pub fn build(release: bool) -> eyre::Result<()> {
    let mut args = vec!["wasi", "build"];
    if release {
        args.push("--release");
    }
    let status =
        Command::new("cargo").args(&args).status().expect("Failed to execute cargo wasi build");

    if !status.success() {
        return Err(eyre::eyre!("cargo wasi build failed"));
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

    println!("xxfunc build completed successfully");
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
        println!("xxfunc deploy completed successfully");
        println!("Response: {}", response.text()?);
        Ok(())
    } else {
        println!("Failed with status: {}", response.status());
        Err(eyre::eyre!("xxfunc deploy failed"))
    }
}

pub fn start(url: &str, module_name: &str) -> eyre::Result<()> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&format!("{}/start", url))
        .json(&json!({
            "module": module_name
        }))
        .send()
        .expect("Failed to send start request");

    if response.status().is_success() {
        println!("xxfunc start completed successfully");
        Ok(())
    } else {
        println!("Failed with status: {}", response.status());
        Err(eyre::eyre!("xxfunc start failed"))
    }
}

pub fn stop(url: &str, module_name: &str) -> eyre::Result<()> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&format!("{}/stop", url))
        .json(&json!({
            "module": module_name
        }))
        .send()
        .expect("Failed to send stop request");

    if response.status().is_success() {
        println!("xxfunc stop completed successfully");
        Ok(())
    } else {
        println!("Failed with status: {}", response.status());
        Err(eyre::eyre!("xxfunc stop failed"))
    }
}

// Helper function to get the project name from Cargo.toml
fn get_project_name() -> eyre::Result<String> {
    let cargo_toml: Value = toml::from_str(&fs::read_to_string("Cargo.toml")?)?;

    let name = cargo_toml
        .get("package")
        .and_then(|p| p.as_table())
        .ok_or_else(|| eyre!("Invalid or missing [package] section in Cargo.toml"))?
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| eyre!("Missing or invalid 'name' field in [package]"))?;

    Ok(name.to_string())
}
