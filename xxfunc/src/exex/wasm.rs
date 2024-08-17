use std::collections::HashMap;

use base64::{prelude::BASE64_STANDARD, Engine};
use jsonrpsee::{core::RpcResult, tracing::info};
use reth_exex::{ExExContext, ExExEvent, ExExNotification};
use reth_node_api::FullNodeComponents;
use reth_tracing::tracing::error;
use tokio::sync::{mpsc, oneshot};
use wasi_common::{sync::WasiCtxBuilder, WasiCtx};
use wasmtime::{Engine as WasmTimeEngine, Linker, Memory, Module, Store, TypedFunc};

use crate::exex::rpc::rpc_internal_error_format;

use super::rpc::RpcMessage;

type AllocParams = (i64,);
type AllocReturn = i64;
type NotificationParams = (i64, i64);
type NotificationReturn = i64;

pub struct WasmExEx<Node: FullNodeComponents> {
    ctx: ExExContext<Node>,
    rpc_messages: mpsc::UnboundedReceiver<(RpcMessage, oneshot::Sender<RpcResult<()>>)>,
    engine: WasmTimeEngine,
    linker: Linker<WasiCtx>,
    installed_exexes: HashMap<String, Module>,
    running_exexes: HashMap<String, RunningExEx>,
}

impl<Node: FullNodeComponents> WasmExEx<Node> {
    pub fn new(
        ctx: ExExContext<Node>,
        rpc_messages: mpsc::UnboundedReceiver<(RpcMessage, oneshot::Sender<RpcResult<()>>)>,
    ) -> eyre::Result<Self> {
        let engine = WasmTimeEngine::default();
        let mut linker = Linker::<WasiCtx>::new(&engine);
        wasi_common::sync::add_to_linker(&mut linker, |s| s)
            .map_err(|err| eyre::eyre!("failed to add WASI: {err}"))?;

        Ok(Self {
            ctx,
            rpc_messages,
            engine,
            linker,
            installed_exexes: HashMap::new(),
            running_exexes: HashMap::new(),
        })
    }

    pub async fn start(mut self) -> eyre::Result<()> {
        loop {
            tokio::select! {
            Some(notification) = self.ctx.notifications.recv() => {
                    self.handle_notification(notification).await?
            }
            Some((rpc_message, tx)) = self.rpc_messages.recv() => {
                let _ = tx
                    .send(self.handle_rpc_message(rpc_message).await)
                    .inspect_err(|err| error!("failed to send response: {err:?}"));
                },
            }
        }
    }

    async fn handle_notification(&mut self, notification: ExExNotification) -> eyre::Result<()> {
        let committed_chain_tip = notification.committed_chain().map(|chain| chain.tip().number);

        for exex in self.running_exexes.values_mut() {
            if let Err(err) = exex.process_notification(&notification) {
                error!(name = %exex.name, %err, "failed to process notification")
            }
        }

        if let Some(tip) = committed_chain_tip {
            self.ctx.events.send(ExExEvent::FinishedHeight(tip))?;
        }

        info!(committed_chain_tip = committed_chain_tip, "committed chain tip",);

        Ok(())
    }

    async fn handle_rpc_message(&mut self, rpc_message: RpcMessage) -> RpcResult<()> {
        match &rpc_message {
            RpcMessage::Install(name, wasm_base64) => {
                let wasm_bytescode = BASE64_STANDARD
                    .decode(wasm_base64)
                    .map_err(|err| rpc_internal_error_format!("failed to decode base64: {err}"))?;
                let module = Module::new(&self.engine, wasm_bytescode).map_err(|err| {
                    rpc_internal_error_format!("failed to create module for {name}: {err}")
                })?;
                self.installed_exexes.insert(name.clone(), module);
            }
            RpcMessage::Start(name) => {
                let module = self
                    .installed_exexes
                    .get(name)
                    .ok_or_else(|| rpc_internal_error_format!("ExEx {name} not installed"))?;

                let exex = RunningExEx::new(name.clone(), &self.engine, module, &self.linker)
                    .map_err(|err| {
                        rpc_internal_error_format!("failed to create exex for {name}: {err}")
                    })?;

                self.running_exexes.insert(name.clone(), exex);
            }
            RpcMessage::Stop(name) => {
                self.running_exexes.remove(name).ok_or_else(|| {
                    rpc_internal_error_format!("no running exex found for {name}")
                })?;
            }
        }

        info!(%rpc_message, "Handled RPC message");

        Ok(())
    }
}

pub struct RunningExEx {
    pub name: String,
    pub store: Store<WasiCtx>,
    pub memory: Memory,
    pub alloc_func: TypedFunc<AllocParams, AllocReturn>,
    pub process_func: TypedFunc<NotificationParams, NotificationReturn>,
}

impl RunningExEx {
    /// Creates a new instance of a running WASM-powered ExEx.
    ///
    /// Initializes a WASM instance with WASI support, prepares the memory and the typed
    /// functions.
    pub fn new(
        name: String,
        engine: &WasmTimeEngine,
        module: &Module,
        linker: &Linker<WasiCtx>,
    ) -> eyre::Result<Self> {
        let wasi = WasiCtxBuilder::new().build();
        let mut store = Store::new(engine, wasi);

        let instance = linker
            .instantiate(&mut store, module)
            .map_err(|err| eyre::eyre!("failed to instantiate: {err}"))?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| eyre::eyre!("failed to get memory"))?;
        let alloc_func = instance
            .get_typed_func::<AllocParams, AllocReturn>(&mut store, "alloc")
            .map_err(|err| eyre::eyre!("failed to get alloc func: {err}"))?;
        let process_func = instance
            .get_typed_func::<NotificationParams, NotificationReturn>(&mut store, "process")
            .map_err(|err| eyre::eyre!("failed to get process func: {err}"))?;

        Ok(Self { name, store, memory, alloc_func, process_func })
    }

    /// Processes an [`ExExNotification`] using the WASM instance.
    // TODO(alexey): we can probably use shared memory here to avoid copying the data into every
    // WASM instance memory. I tried it for a while and it didn't work straight away. Maybe we can
    // share a portion of linear memory, but the rest is up to the WASM instance to manage?
    pub fn process_notification(
        &mut self,
        notification: &reth_exex::ExExNotification,
    ) -> eyre::Result<()> {
        // TODO(alexey): serialize to bincode or just cast to bytes directly. Can't do it now
        // because `ExExNotification` can't be used inside WASM.
        let serialized_notification =
            // Can't even do JSON encode of a full struct smh, "key must be a string"
            serde_json::to_vec(&notification.committed_chain().map(|chain| chain.tip().header.clone()))?;

        // Allocate memory for the notification.
        let data_size = serialized_notification.len() as i64;
        let data_ptr = self
            .alloc_func
            .call(&mut self.store, (data_size,))
            .map_err(|err| eyre::eyre!("failed to call alloc func: {err}"))?;

        // Write the notification to the allocated memory.
        self.memory.write(&mut self.store, data_ptr as usize, &serialized_notification)?;

        // Call the notification function that will read the allocated memoyry.
        let output = self
            .process_func
            .call(&mut self.store, (data_ptr, data_size))
            .map_err(|err| eyre::eyre!("failed to call notification func: {err}"))?;

        info!(target: "wasm", name = %self.name, ?data_ptr, ?data_size, ?output, "Processed notification");

        Ok(())
    }
}
