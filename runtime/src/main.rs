mod wasm;
mod proto {
    tonic::include_proto!("execute_service");
}

use eyre::Result;
use proto::execute_server::Execute;
use proto::execute_server::ExecuteServer;
use proto::{ExecuteRequest, ExecuteResponse};
use tonic::{transport::Server, Request, Response, Status};
use wasm::Runtime;

pub struct ExecuteService {
    runtime: Runtime,
}

#[tonic::async_trait]
impl Execute for ExecuteService {
    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        let req = request.into_inner();
        // TODO: Implement the actual execution logic here
        let result = format!("Executed {} with {} bytes of data", req.name, req.data.len());
        Ok(Response::new(ExecuteResponse { result }))
    }
}

pub async fn run_server() -> Result<()> {
    let addr = "[::1]:50051".parse()?;

    let runtime = Runtime::new()?;
    let service = ExecuteService { runtime };

    Server::builder().add_service(ExecuteServer::new(service)).serve(addr).await?;

    Ok(())
}
