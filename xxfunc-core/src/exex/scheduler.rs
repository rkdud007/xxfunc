use eyre::Result;
use reth_exex::ExExContext;
use reth_node_api::FullNodeComponents;
use xxfunc_db::ModuleDatabase;

pub struct Scheduler<N: FullNodeComponents> {
    // handle to the runtime where tasks can be queued
    // runtime : Runtime
    db: ModuleDatabase,
    exex_ctx: ExExContext<N>,
}

impl<N: FullNodeComponents> Scheduler<N> {
    pub fn new(exex_ctx: ExExContext<N>) -> Result<Self> {
        let db = ModuleDatabase::open("./")?;
        Ok(Self { exex_ctx, db })
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

    // retrieves all the active (ie started) modules from the database
    async fn get_active_modules(&self) -> Vec<()> {
        Default::default()
    }
}
