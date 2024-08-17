mod wasm;
mod proto {
    tonic::include_proto!("execute_service");
}

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

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "[::1]:50051".parse()?;

    let service = ExecuteService::new()?;
    Server::builder().add_service(ExecuteServer::new(service)).serve(addr).await?;

    Ok(())
}
