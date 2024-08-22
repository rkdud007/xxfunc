use std::sync::Arc;

use eyre::Result;
use futures::channel::oneshot;
use reth_exex_types::ExExNotification;
use xxfunc_db::{ModuleDatabase, ModuleId};

pub trait Runtime {
    type ExecutionResult;

    fn new(db: ModuleDatabase) -> Result<Self>
    where
        Self: std::marker::Sized;

    fn spawn(
        &self,
        module_id: ModuleId,
        exex_notification: Arc<ExExNotification>,
    ) -> JoinHandle<Result<Self::ExecutionResult>>;

    fn get_db(&self) -> &ModuleDatabase;

    fn wake(&self);
}

#[derive(Debug)]
pub struct JoinHandle<T>(pub oneshot::Receiver<T>);

impl<T> std::future::Future for JoinHandle<T> {
    type Output = Result<T, oneshot::Canceled>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        std::pin::Pin::new(&mut self.0).poll(cx)
    }
}
