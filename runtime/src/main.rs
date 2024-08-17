mod wasm;
mod proto {
    tonic::include_proto!("execute_service");
}

use clap::Parser;
use eyre::Result;
use proto::execute_server::Execute;
use proto::execute_server::ExecuteServer;
use proto::{ExecuteRequest, ExecuteResponse};
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;
use wasm::Runtime;

pub struct ExecuteService {
    runtime: Runtime,
}

impl ExecuteService {
    pub fn new() -> Result<Self> {
        let runtime = Runtime::new()?;
        Ok(ExecuteService { runtime })
    }
}

#[tonic::async_trait]
impl Execute for ExecuteService {
    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        info!("Executing function");
        let _ = request.into_inner();
        // TODO: Implement the actual execution logic here
        Ok(Response::new(ExecuteResponse {}))
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on. If not specified, a random port will be used.
    #[arg(short, long, default_value_t = 0)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let addr = format!("[::1]:{}", args.port).parse()?;

    let service = ExecuteService::new()?;
    Server::builder().add_service(ExecuteServer::new(service)).serve(addr).await?;

    Ok(())
}
