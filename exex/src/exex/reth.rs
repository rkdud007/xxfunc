use std::{sync::Arc, time::Duration};

use reth_execution_types::Chain;
use reth_exex::ExExNotification;
use tokio::{sync::mpsc, time};
use tracing::info;

use super::{
    rpc::{ExExRpcExt, ExExRpcExtApiServer},
    scheduler::Scheduler,
};

pub fn init_reth() -> eyre::Result<()> {
    reth::cli::Cli::parse_args().run(|builder, _| async move {
        let (rpc_tx, _) = mpsc::unbounded_channel();

        // create fake notification channel to controll notification sender
        let (notification_sender, notification_receiver) = mpsc::channel(100);

        let handle = builder
            .node(reth_node_ethereum::EthereumNode::default())
            .extend_rpc_modules(move |ctx| {
                ctx.modules.merge_configured(ExExRpcExt { to_exex: rpc_tx }.into_rpc())?;
                Ok(())
            })
            .install_exex("xx", |mut ctx| async {
                // override notification receiver
                ctx.notifications = notification_receiver;

                Ok(Scheduler::new(ctx)?.start())
            })
            .launch()
            .await?;

        // Start a task to send mock notifications every 10 seconds
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                let notification = ExExNotification::ChainCommitted {
                    new: Arc::new(Chain::from_block(
                        Default::default(),
                        Default::default(),
                        Default::default(),
                    )),
                };
                if let Err(e) = notification_sender.send(notification).await {
                    eprintln!("Failed to send notification: {}", e);
                    break;
                }
                info!("📢 Sent mock exex notification");
            }
        });

        handle.wait_for_node_exit().await
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn run_reth() {
        let _ = super::init_reth();
    }
}
