use eyre::Result;
use tokio::sync::mpsc;

use super::rpc::{ExExRpcExt, ExExRpcExtApiServer};

pub fn init_reth() -> eyre::Result<()> {
    reth::cli::Cli::parse_args().run(|builder, _| async move {
        let (rpc_tx, rpc_rx) = mpsc::unbounded_channel();

        let handle = builder
            .node(reth_node_ethereum::EthereumNode::default())
            .extend_rpc_modules(move |ctx| {
                ctx.modules.merge_configured(ExExRpcExt { to_exex: rpc_tx }.into_rpc())?;
                Ok(())
            })
            .install_exex(
                "xx",
                |mut ctx| async move { Result::Ok(async { Result::Ok(()) }) }, // |mut ctx| async move { Ok(WasmExEx::new(ctx, rpc_rx)?.start()) },
            )
            .launch()
            .await?;

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
