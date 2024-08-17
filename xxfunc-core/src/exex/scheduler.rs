use eyre::Result;
use reth_exex::ExExContext;
use reth_node_api::FullNodeComponents;

pub struct Scheduler<N: FullNodeComponents> {
    // handle to the runtime where tasks can be queued
    // runtime : Runtime
    exex_ctx: ExExContext<N>,
}

impl<N: FullNodeComponents> Scheduler<N> {
    pub fn new(exex_ctx: ExExContext<N>) -> Self {
        Self { exex_ctx }
    }

    pub async fn start(mut self) -> Result<()> {
        loop {
            tokio::select! {
                Some(notification) = self.exex_ctx.notifications.recv() => {
                    println!("handle exex notification")
                }
            }
        }
    }
}
