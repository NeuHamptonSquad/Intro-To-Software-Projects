use std::{
    fmt::Debug,
    io::{ErrorKind, LineWriter, Write},
    mem::ManuallyDrop,
    sync::{Arc, atomic::AtomicU16},
    thread::JoinHandle,
};

use axum::{
    Router,
    body::Bytes,
    extract::{
        State, WebSocketUpgrade,
        ws::{self, WebSocket},
    },
    http::{HeaderValue, Method},
    response::Html,
    routing::get,
};
use futures_util::TryFutureExt;
use godot::{classes::Object, prelude::*};
use tokio::sync::{Mutex, broadcast};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::{Level, dispatcher, span};
use tracing_subscriber::fmt::MakeWriter;

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
    #[func]
    pub fn span(&mut self, location: GString, varargs: GString) -> u64 {
        let entered = ManuallyDrop::new(
            tracing::span!(
                Level::INFO,
                "godot",
                location = %location,
                args = %varargs
            )
            .entered(),
        );
        entered.id().unwrap().into_u64()
    }
    #[func]
    pub fn exit_span(&mut self, span: u64) {
        dispatcher::get_default(|dispatcher| {
            let span_id = span::Id::from_u64(span);
            dispatcher.exit(&span_id);
            if !dispatcher.try_close(span_id) {
                tracing::warn!("Failed to close godot span");
            }
        });
    }
}

#[derive(Clone)]
pub(crate) enum LogServerEvent {
    Log(Arc<str>),
    NewInstance(u16),
}

pub(crate) struct LogServerMakeWriter;

impl<'a> MakeWriter<'a> for LogServerMakeWriter {
    type Writer = LineWriter<LogServerWriter>;

    fn make_writer(&'a self) -> Self::Writer {
        LineWriter::new(LogServerWriter(Level::INFO))
    }

    fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        LineWriter::new(LogServerWriter(*meta.level()))
    }
}

pub(crate) struct LogServerWriter(Level);

impl Write for LogServerWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let buf =
            std::str::from_utf8(buf).map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;
        match self.0 {
            Level::ERROR => {
                godot::global::push_error(&[Variant::from(strip_ansi::strip_ansi(buf))])
            }
            Level::WARN => {
                godot::global::push_warning(&[Variant::from(strip_ansi::strip_ansi(buf))])
            }
            _ => {}
        }
        if let Some(log_server) = LOG_SERVER.get() {
            let _ = log_server.log_sender.send(LogServerEvent::Log(buf.into()));
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(crate) struct LogServer {
    handle: axum_server::Handle,
    log_sender: broadcast::Sender<LogServerEvent>,
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
        let (log_sender, log_receiver) = broadcast::channel(10);
        let join_handle = std::thread::spawn({
            let handle = handle.clone();
            let log_sender = log_sender.clone();
            move || start_log_server(handle, log_sender, log_receiver)
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

#[derive(Clone)]
struct LogServerState {
    log_sender: broadcast::Sender<LogServerEvent>,
    port_serial: Arc<AtomicU16>,
    first_receiver: Arc<Mutex<Option<broadcast::Receiver<LogServerEvent>>>>,
}

impl LogServerState {
    fn new(
        log_sender: broadcast::Sender<LogServerEvent>,
        log_receiver: broadcast::Receiver<LogServerEvent>,
    ) -> Self {
        Self {
            first_receiver: Arc::new(Mutex::new(Some(log_receiver))),
            log_sender,
            port_serial: Arc::new(8001.into()),
        }
    }
}

#[tokio::main(flavor = "current_thread")]
pub(crate) async fn start_log_server(
    handle: axum_server::Handle,
    log_sender: broadcast::Sender<LogServerEvent>,
    log_receiver: broadcast::Receiver<LogServerEvent>,
) {
    let app = Router::new()
        .route("/", get(async || Html(include_str!("log_server.html"))))
        .route("/ping", get(async || "pong"))
        .route("/new_instance", get(new_instance))
        .route("/ws", get(log_ws))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET])
                .allow_origin(AllowOrigin::exact(HeaderValue::from_static(
                    "http://localhost:8000",
                ))),
        )
        .with_state(LogServerState::new(log_sender, log_receiver));

    let socket_addr = if let Ok(response) = reqwest::get("http://localhost:8000/new_instance")
        .and_then(|resp| resp.text())
        .await
    {
        let port: u16 = response.parse().unwrap();
        // There is an editor running, we are not the
        // main server.
        ([127, 0, 0, 1], port).into()
    } else {
        // We are the main server
        ([127, 0, 0, 1], 8000).into()
    };

    axum_server::bind(socket_addr)
        .handle(handle)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn new_instance(State(log_server_state): State<LogServerState>) -> String {
    let new_instance_port = log_server_state
        .port_serial
        .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    // You cannot simultaniously spawn 1000 instances of the game.
    // Cope, seath, cry about it.
    if new_instance_port == 8999 {
        log_server_state
            .port_serial
            .store(8001, std::sync::atomic::Ordering::Release);
    }
    let _ = log_server_state
        .log_sender
        .send(LogServerEvent::NewInstance(new_instance_port));
    new_instance_port.to_string()
}

async fn log_ws(
    State(log_server_state): State<LogServerState>,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    let log_event_receiver = log_server_state
        .first_receiver
        .lock()
        .await
        .take()
        .unwrap_or_else(|| log_server_state.log_sender.subscribe());
    ws.on_upgrade(move |ws| handle_socket(ws, log_event_receiver))
}

async fn handle_socket(
    mut socket: WebSocket,
    mut log_event_receiver: broadcast::Receiver<LogServerEvent>,
) {
    loop {
        tokio::select! {
            log_event = log_event_receiver.recv() => {
                match log_event {
                    Ok(log_event) => {
                        match log_event {
                            LogServerEvent::Log(log) => {
                                let html = ansi_to_html::convert(&log).unwrap();
                                if let Err(_) = socket.send(ws::Message::Text(html.into())).await {
                                    return;
                                };
                            }
                            LogServerEvent::NewInstance(port) => {
                                let port = port.to_le_bytes();
                                if let Err(_) = socket.send(ws::Message::Binary(Bytes::from_owner(port))).await {
                                    return;
                                };
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(missed_logs)) => {
                        let missed_log_message = format!("Uh oh, you missed {} logs", missed_logs);
                        if let Err(_) = socket.send(ws::Message::Text(missed_log_message.into())).await {
                            return;
                        };
                    }
                    Err(broadcast::error::RecvError::Closed) => return,
                }
            }
            msg = socket.recv() => {
                if msg.and_then(|m| m.ok()).is_none() {
                    return;
                }
            }
        }
    }
}
