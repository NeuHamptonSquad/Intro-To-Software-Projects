use std::{
    fmt::Debug,
    io::{ErrorKind, Write},
    sync::Arc,
    thread::JoinHandle,
};

use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{self, WebSocket},
    },
    response::Html,
    routing::get,
};
use godot::{classes::Object, prelude::*};
use tokio::sync::broadcast;

use crate::LOG_SERVER;

#[derive(GodotClass)]
#[class(init, base=Object)]
pub struct Logger {
    base: Base<Object>,
}

#[godot_api]
impl Logger {
    #[func]
    pub fn error(&mut self, varargs: GString) {
        tracing::error!(
            target: "godot",
            "{}", varargs
        );
    }
    #[func]
    pub fn warn(&mut self, varargs: GString) {
        tracing::warn!(
            target: "godot",
            "{}", varargs
        );
    }
    #[func]
    pub fn info(&mut self, varargs: GString) {
        tracing::info!(
            target: "godot",
            "{}", varargs
        );
    }
    #[func]
    pub fn debug(&mut self, varargs: GString) {
        tracing::debug!(
            target: "godot",
            "{}", varargs
        );
    }
    #[func]
    pub fn trace(&mut self, varargs: GString) {
        tracing::trace!(
            target: "godot",
            "{}", varargs
        );
    }
}

pub(crate) struct LogServerWriter;

impl Write for LogServerWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let buf =
            std::str::from_utf8(buf).map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;
        if let Some(log_server) = LOG_SERVER.get() {
            let _ = log_server.log_sender.send(buf.into());
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(crate) struct LogServer {
    handle: axum_server::Handle,
    log_sender: broadcast::Sender<Arc<str>>,
    join_handle: JoinHandle<()>,
}

impl Debug for LogServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogServer").finish_non_exhaustive()
    }
}

impl LogServer {
    pub(crate) fn new() -> Self {
        let handle = axum_server::Handle::new();
        let (log_sender, _) = broadcast::channel(10);
        let join_handle = std::thread::spawn({
            let handle = handle.clone();
            let log_sender = log_sender.clone();
            move || start_log_server(handle, log_sender)
        });
        Self {
            handle,
            log_sender,
            join_handle,
        }
    }

    pub(crate) fn join(&self) {
        self.handle.graceful_shutdown(None);
        while !self.join_handle.is_finished() {
            std::hint::spin_loop();
        }
    }
}

#[tokio::main(flavor = "current_thread")]
pub(crate) async fn start_log_server(
    handle: axum_server::Handle,
    log_sender: broadcast::Sender<Arc<str>>,
) {
    let app = Router::new()
        .route("/", get(async || Html(include_str!("log_server.html"))))
        .route("/ping", get(async || "pong"))
        .route("/ws", get(log_ws))
        .with_state(log_sender);

    axum_server::bind(([127, 0, 0, 1], 3000).into())
        .handle(handle)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn log_ws(
    State(log_sender): State<broadcast::Sender<Arc<str>>>,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(move |ws| handle_socket(ws, log_sender.subscribe()))
}

async fn handle_socket(mut socket: WebSocket, mut log_receiver: broadcast::Receiver<Arc<str>>) {
    loop {
        tokio::select! {
             log = log_receiver.recv() => {
                let Ok(log) = log else {
                    return;
                };
                let html = ansi_to_html::convert(&log).unwrap();
                if let Err(_) = socket.send(ws::Message::Text(html.into())).await {
                    return;
                };
            }
            msg = socket.recv() => {
                if msg.and_then(|m| m.ok()).is_none() {
                    return;
                }
            }
        }
    }
}
