use std::sync::Arc;

use eyre::Result;
use jsonrpsee::tracing::info;
use reth_exex::{ExExContext, ExExNotification};
use reth_node_api::FullNodeComponents;
use xxfunc_db::{ModuleDatabase, ModuleId, ModuleState};
use xxfunc_runtime::runtime::Runtime;

pub struct Scheduler<N: FullNodeComponents> {
    // handle to the runtime where tasks can be queued
    runtime: Runtime,
    db: ModuleDatabase,
    exex_ctx: ExExContext<N>,
}

impl<N: FullNodeComponents> Scheduler<N> {
    pub fn new(exex_ctx: ExExContext<N>) -> Result<Self> {
        let db = ModuleDatabase::open("module.db")?;
        let runtime = Runtime::new(db.clone())?;

        Ok(Self { runtime, exex_ctx, db })
    }

    pub async fn start(mut self) -> Result<()> {
        loop {
            tokio::select! {
                Some(notification) = self.exex_ctx.notifications.recv() => {
                    self.handle_notification(notification).await?;
                }
            }
        }
    }

    async fn handle_notification(&self, notification: ExExNotification) -> Result<()> {
        let count = self.spawn_tasks(notification).await?;
        info!(%count, "Scheduled tasks.");
        Ok(())
    }

    // spawn tasks on the runtime and return the number of tasks spawned
    async fn spawn_tasks(&self, notification: ExExNotification) -> Result<usize> {
        let exex_notification = Arc::new(notification);
        let modules = self.get_active_modules()?;

        let mut count = 0;
        for id in modules {
            let result = self.runtime.spawn(id, Arc::clone(&exex_notification));

            count += 1;
        }

        Ok(count)
    }

    // retrieves all the active (ie started) modules from the database
    fn get_active_modules(&self) -> Result<Vec<ModuleId>> {
        self.db.get_modules_by_state(ModuleState::Started)
    }
}
