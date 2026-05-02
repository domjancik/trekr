use crate::actions::{ActionSource, AppAction};
use crate::app::{App, AppControl, ClientUiState, rect_contains};
use crate::diagnostics;
use crate::link::LinkSnapshot;
use crate::mapping::MappingEntry;
use crate::midi_io::MidiDeviceCatalog;
use crate::pages::{MappingField, MidiIoListFocus, RoutingField};
use crate::project::Project;
use crate::theme::{ThemePreset, theme};
use crate::timeline_fx::{TimelineContext, TimelineFxField};
use if_addrs::{IfAddr, get_if_addrs};
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::rect::Rect;
use sdl3::render::Canvas;
use sdl3::video::Window;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const SNAPSHOT_BROADCAST_INTERVAL: Duration = Duration::from_millis(100);
const HEADLESS_STATUS_INTERVAL: Duration = Duration::from_millis(500);
const IO_POLL_INTERVAL: Duration = Duration::from_millis(16);
const SDL_CLIENT_FRAME_INTERVAL: Duration = Duration::from_millis(16);
const DISCOVERY_QUERY_INTERVAL: Duration = Duration::from_millis(1000);
const DISCOVERY_STALE_AFTER: Duration = Duration::from_millis(4000);
const DISCOVERY_UDP_PORT: u16 = 8789;
const DISCOVERY_PROTOCOL_VERSION: u32 = 1;
const CONNECT_PROBE_TIMEOUT: Duration = Duration::from_millis(180);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionCommand {
    TogglePlayback,
    ToggleRecording,
    ToggleGlobalLoop,
    ResetGlobalLoop,
    SelectNextTrack,
    SelectPreviousTrack,
    ToggleCurrentTrackArm,
    ToggleCurrentTrackMute,
    ToggleCurrentTrackSolo,
    ToggleCurrentTrackPassthrough,
}

impl SessionCommand {
    pub const ALL: [SessionCommand; 10] = [
        SessionCommand::TogglePlayback,
        SessionCommand::ToggleRecording,
        SessionCommand::ToggleGlobalLoop,
        SessionCommand::ResetGlobalLoop,
        SessionCommand::SelectNextTrack,
        SessionCommand::SelectPreviousTrack,
        SessionCommand::ToggleCurrentTrackArm,
        SessionCommand::ToggleCurrentTrackMute,
        SessionCommand::ToggleCurrentTrackSolo,
        SessionCommand::ToggleCurrentTrackPassthrough,
    ];

    pub fn action(self) -> AppAction {
        match self {
            SessionCommand::TogglePlayback => AppAction::TogglePlayback,
            SessionCommand::ToggleRecording => AppAction::ToggleRecording,
            SessionCommand::ToggleGlobalLoop => AppAction::ToggleGlobalLoop,
            SessionCommand::ResetGlobalLoop => AppAction::ResetGlobalLoop,
            SessionCommand::SelectNextTrack => AppAction::SelectNextTrack,
            SessionCommand::SelectPreviousTrack => AppAction::SelectPreviousTrack,
            SessionCommand::ToggleCurrentTrackArm => AppAction::ToggleCurrentTrackArm,
            SessionCommand::ToggleCurrentTrackMute => AppAction::ToggleCurrentTrackMute,
            SessionCommand::ToggleCurrentTrackSolo => AppAction::ToggleCurrentTrackSolo,
            SessionCommand::ToggleCurrentTrackPassthrough => {
                AppAction::ToggleCurrentTrackPassthrough
            }
        }
    }

    pub fn parse_token(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "space" | "play" | "toggle-playback" | "toggle_playback" => {
                Some(SessionCommand::TogglePlayback)
            }
            "record" | "toggle-recording" | "toggle_recording" | "r" => {
                Some(SessionCommand::ToggleRecording)
            }
            "loop" | "toggle-loop" | "toggle_loop" | "g" => Some(SessionCommand::ToggleGlobalLoop),
            "reset-loop" | "reset_loop" | "home" => Some(SessionCommand::ResetGlobalLoop),
            "next-track" | "next_track" | "right" => Some(SessionCommand::SelectNextTrack),
            "prev-track" | "previous-track" | "previous_track" | "left" => {
                Some(SessionCommand::SelectPreviousTrack)
            }
            "arm" | "toggle-arm" | "toggle_arm" | "a" => {
                Some(SessionCommand::ToggleCurrentTrackArm)
            }
            "mute" | "toggle-mute" | "toggle_mute" | "m" => {
                Some(SessionCommand::ToggleCurrentTrackMute)
            }
            "solo" | "toggle-solo" | "toggle_solo" | "s" => {
                Some(SessionCommand::ToggleCurrentTrackSolo)
            }
            "passthrough" | "toggle-passthrough" | "toggle_passthrough" | "i" => {
                Some(SessionCommand::ToggleCurrentTrackPassthrough)
            }
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SessionCommand::TogglePlayback => "Space/play",
            SessionCommand::ToggleRecording => "R/record",
            SessionCommand::ToggleGlobalLoop => "G/loop",
            SessionCommand::ResetGlobalLoop => "Home/reset-loop",
            SessionCommand::SelectNextTrack => "Right/next-track",
            SessionCommand::SelectPreviousTrack => "Left/prev-track",
            SessionCommand::ToggleCurrentTrackArm => "A/arm",
            SessionCommand::ToggleCurrentTrackMute => "M/mute",
            SessionCommand::ToggleCurrentTrackSolo => "S/solo",
            SessionCommand::ToggleCurrentTrackPassthrough => "I/passthrough",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub revision: u64,
    pub connected_clients: usize,
    pub project: Project,
    pub mappings: Vec<MappingEntry>,
    pub midi_devices: MidiDeviceCatalog,
    pub link_snapshot: LinkSnapshot,
    pub transport_ticks: u64,
    pub playhead_ticks: u64,
}

#[derive(Debug, Clone)]
pub struct DiscoveryHostConfig {
    pub session_name: String,
    pub host_mode: String,
    pub listen_addr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryAdvertisement {
    pub session_id: String,
    pub session_name: String,
    pub host_name: String,
    pub listen_addr: String,
    pub connect_addrs: Vec<String>,
    pub port: u16,
    pub protocol_version: u32,
    pub host_mode: String,
    pub current_client_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum DiscoveryWireMessage {
    Query {
        protocol_version: u32,
        client_name: String,
    },
    Advertisement {
        session: DiscoveryAdvertisement,
    },
}

#[derive(Debug, Clone)]
struct DiscoveredSession {
    session: DiscoveryAdvertisement,
    preferred_connect_addr: String,
    last_seen_at: Instant,
}

impl DiscoveredSession {
    fn is_compatible(&self) -> bool {
        self.session.protocol_version == DISCOVERY_PROTOCOL_VERSION
    }

    fn subtitle(&self) -> String {
        format!(
            "{} | {} client{} | {}",
            self.preferred_connect_addr,
            self.session.current_client_count,
            if self.session.current_client_count == 1 {
                ""
            } else {
                "s"
            },
            self.session.host_mode
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RemoteUiIntent {
    Action {
        action: AppAction,
    },
    Actions {
        actions: Vec<AppAction>,
    },
    TrackAction {
        track_index: usize,
        action: AppAction,
    },
    BeginDirectMappingInput {
        action: AppAction,
        target_label: String,
        scope_label: String,
        display_scope: Option<String>,
    },
    CaptureDirectMappingKey {
        source_label: String,
    },
    SelectMappingRow {
        index: usize,
    },
    ActivateMappingField {
        index: usize,
        field: MappingField,
        activate: bool,
    },
    CommitMappingTargetLookupLabel {
        label: String,
    },
    AppendMappingTargetLookupText {
        text: String,
    },
    BackspaceMappingTargetLookup,
    CancelMappingTargetLookup,
    SetMidiIoFocus {
        focus: MidiIoListFocus,
    },
    SelectMidiInput {
        index: usize,
    },
    SelectMidiOutput {
        index: usize,
    },
    RoutingAdjustField {
        field: RoutingField,
        delta: i32,
    },
    RoutingActivateField {
        field: RoutingField,
    },
    TimelineFxClick {
        track_index: usize,
        context: TimelineContext,
        row_index: usize,
        field: Option<TimelineFxField>,
        action: Option<AppAction>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ThinClientMessage {
    Hello { client_name: String },
    Command { command: SessionCommand },
    UiIntent { intent: RemoteUiIntent },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HostMessage {
    Snapshot {
        snapshot: SessionSnapshot,
    },
    Ack {
        revision: u64,
        command: SessionCommand,
    },
    Reject {
        message: String,
    },
}

#[derive(Debug, Clone)]
enum ReceivedClientMessage {
    Connected {
        client_id: usize,
    },
    Disconnected {
        client_id: usize,
    },
    Hello {
        client_id: usize,
        client_name: String,
    },
    Command {
        command: SessionCommand,
    },
    UiIntent {
        client_id: usize,
        intent: RemoteUiIntent,
    },
}

#[derive(Debug, Clone)]
struct ClientContext {
    client_name: String,
    ui_state: ClientUiState,
}

struct DiscoveryHostService {
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl DiscoveryHostService {
    fn start(
        config: DiscoveryHostConfig,
        connected_client_count: Arc<AtomicUsize>,
    ) -> Option<Self> {
        let socket = UdpSocket::bind(("0.0.0.0", DISCOVERY_UDP_PORT)).ok()?;
        if socket.set_broadcast(true).is_err()
            || socket.set_read_timeout(Some(IO_POLL_INTERVAL)).is_err()
        {
            return None;
        }

        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            let host_name = local_host_name();
            let port = parse_listen_port(&config.listen_addr);
            let session_id = format!("{host_name}:{port}");
            let connect_addrs = discover_connect_addrs(&config.listen_addr, &host_name);
            let mut buffer = [0_u8; 4096];
            diagnostics::log_info(
                "distributed",
                format!(
                    "discovery host advertisement ready on udp {DISCOVERY_UDP_PORT} for {}",
                    config.listen_addr
                ),
            );

            while !stop_thread.load(Ordering::Relaxed) {
                match socket.recv_from(&mut buffer) {
                    Ok((len, source)) => {
                        let Ok(DiscoveryWireMessage::Query { .. }) =
                            serde_json::from_slice::<DiscoveryWireMessage>(&buffer[..len])
                        else {
                            continue;
                        };
                        let advertisement = DiscoveryAdvertisement {
                            session_id: session_id.clone(),
                            session_name: config.session_name.clone(),
                            host_name: host_name.clone(),
                            listen_addr: config.listen_addr.clone(),
                            connect_addrs: connect_addrs.clone(),
                            port,
                            protocol_version: DISCOVERY_PROTOCOL_VERSION,
                            host_mode: config.host_mode.clone(),
                            current_client_count: connected_client_count.load(Ordering::Relaxed),
                        };
                        let Ok(payload) =
                            serde_json::to_vec(&DiscoveryWireMessage::Advertisement {
                                session: advertisement,
                            })
                        else {
                            continue;
                        };
                        let _ = socket.send_to(&payload, source);
                    }
                    Err(err)
                        if err.kind() == io::ErrorKind::WouldBlock
                            || err.kind() == io::ErrorKind::TimedOut => {}
                    Err(err) => {
                        diagnostics::log_error(
                            "distributed",
                            format!("discovery host error: {err}"),
                        );
                    }
                }
            }
        });

        Some(Self {
            stop,
            thread: Some(thread),
        })
    }
}

impl Drop for DiscoveryHostService {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

struct DiscoveryClient {
    socket: UdpSocket,
    client_name: String,
    last_query_at: Instant,
}

impl DiscoveryClient {
    fn new(client_name: &str) -> io::Result<Self> {
        let socket = UdpSocket::bind(("0.0.0.0", 0))?;
        socket.set_broadcast(true)?;
        socket.set_read_timeout(Some(Duration::from_millis(1)))?;
        Ok(Self {
            socket,
            client_name: client_name.to_string(),
            last_query_at: Instant::now() - DISCOVERY_QUERY_INTERVAL,
        })
    }

    fn request_refresh(&mut self) {
        self.last_query_at = Instant::now() - DISCOVERY_QUERY_INTERVAL;
    }

    fn poll(&mut self, sessions: &mut Vec<DiscoveredSession>) {
        if self.last_query_at.elapsed() >= DISCOVERY_QUERY_INTERVAL {
            let query = DiscoveryWireMessage::Query {
                protocol_version: DISCOVERY_PROTOCOL_VERSION,
                client_name: self.client_name.clone(),
            };
            if let Ok(payload) = serde_json::to_vec(&query) {
                let _ = self.socket.send_to(
                    &payload,
                    SocketAddr::from(([255, 255, 255, 255], DISCOVERY_UDP_PORT)),
                );
            }
            self.last_query_at = Instant::now();
        }

        let mut buffer = [0_u8; 4096];
        loop {
            match self.socket.recv_from(&mut buffer) {
                Ok((len, source)) => {
                    let Ok(DiscoveryWireMessage::Advertisement { session }) =
                        serde_json::from_slice::<DiscoveryWireMessage>(&buffer[..len])
                    else {
                        continue;
                    };
                    let connect_addr =
                        choose_preferred_connect_addr(&session, source.ip().to_string())
                            .unwrap_or_else(|| format!("{}:{}", source.ip(), session.port));
                    upsert_discovered_session(
                        sessions,
                        DiscoveredSession {
                            session,
                            preferred_connect_addr: connect_addr,
                            last_seen_at: Instant::now(),
                        },
                    );
                }
                Err(err)
                    if err.kind() == io::ErrorKind::WouldBlock
                        || err.kind() == io::ErrorKind::TimedOut =>
                {
                    break;
                }
                Err(err) => {
                    diagnostics::log_error("distributed", format!("discovery client error: {err}"));
                    break;
                }
            }
        }

        sessions.retain(|session| session.last_seen_at.elapsed() <= DISCOVERY_STALE_AFTER);
        sessions.sort_by(|left, right| {
            left.session
                .session_name
                .cmp(&right.session.session_name)
                .then(
                    left.preferred_connect_addr
                        .cmp(&right.preferred_connect_addr),
                )
        });
    }
}

fn upsert_discovered_session(sessions: &mut Vec<DiscoveredSession>, discovered: DiscoveredSession) {
    if let Some(existing) = sessions
        .iter_mut()
        .find(|entry| entry.session.session_id == discovered.session.session_id)
    {
        *existing = discovered;
    } else {
        sessions.push(discovered);
    }
}

pub struct SessionServer {
    command_rx: Receiver<ReceivedClientMessage>,
    clients: Arc<Mutex<HashMap<usize, Sender<HostMessage>>>>,
    connected_client_count: Arc<AtomicUsize>,
    contexts: HashMap<usize, ClientContext>,
    revision: u64,
    last_snapshot_json: Option<String>,
    last_snapshot_broadcast_at: Instant,
    _discovery_host_service: Option<DiscoveryHostService>,
    _accept_thread: thread::JoinHandle<()>,
}

impl SessionServer {
    pub fn bind(listen_addr: &str, discovery: DiscoveryHostConfig) -> io::Result<Self> {
        let listener = TcpListener::bind(listen_addr)?;
        let (command_tx, command_rx) = mpsc::channel();
        let clients = Arc::new(Mutex::new(HashMap::<usize, Sender<HostMessage>>::new()));
        let connected_client_count = Arc::new(AtomicUsize::new(0));
        let accept_clients = Arc::clone(&clients);
        let next_client_id = Arc::new(AtomicUsize::new(1));
        let accept_next_client_id = Arc::clone(&next_client_id);
        let accept_thread = thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else {
                    diagnostics::log_error("distributed", "listener incoming stream failed");
                    continue;
                };
                let client_id = accept_next_client_id.fetch_add(1, Ordering::Relaxed);
                let Ok(write_stream) = stream.try_clone() else {
                    diagnostics::log_error(
                        "distributed",
                        format!("failed to clone stream for client {client_id}"),
                    );
                    continue;
                };
                let (outbound_tx, outbound_rx) = mpsc::channel::<HostMessage>();
                if let Ok(mut guard) = accept_clients.lock() {
                    guard.insert(client_id, outbound_tx);
                }
                diagnostics::log_info("distributed", format!("client {client_id} connected"));
                let _ = command_tx.send(ReceivedClientMessage::Connected { client_id });
                spawn_client_writer(write_stream, outbound_rx);
                spawn_client_reader(stream, client_id, command_tx.clone());
            }
        });

        Ok(Self {
            command_rx,
            clients,
            connected_client_count: Arc::clone(&connected_client_count),
            contexts: HashMap::new(),
            revision: 0,
            last_snapshot_json: None,
            last_snapshot_broadcast_at: Instant::now() - SNAPSHOT_BROADCAST_INTERVAL,
            _discovery_host_service: DiscoveryHostService::start(discovery, connected_client_count),
            _accept_thread: accept_thread,
        })
    }

    pub fn service_app(&mut self, app: &mut App) {
        let received_messages: Vec<_> = self.command_rx.try_iter().collect();
        for received in received_messages {
            match received {
                ReceivedClientMessage::Connected { client_id } => {
                    self.contexts
                        .entry(client_id)
                        .or_insert_with(|| ClientContext {
                            client_name: format!("client-{client_id}"),
                            ui_state: ClientUiState::default(),
                        });
                    self.connected_client_count
                        .store(self.contexts.len(), Ordering::Relaxed);
                }
                ReceivedClientMessage::Disconnected { client_id } => {
                    self.contexts.remove(&client_id);
                    if let Ok(mut guard) = self.clients.lock() {
                        guard.remove(&client_id);
                    }
                    self.connected_client_count
                        .store(self.contexts.len(), Ordering::Relaxed);
                    diagnostics::log_info(
                        "distributed",
                        format!("client {client_id} disconnected"),
                    );
                }
                ReceivedClientMessage::Hello {
                    client_id,
                    client_name,
                } => {
                    let initial_name = client_name.clone();
                    self.contexts
                        .entry(client_id)
                        .or_insert_with(|| ClientContext {
                            client_name: initial_name,
                            ui_state: ClientUiState::default(),
                        })
                        .client_name = client_name;
                }
                ReceivedClientMessage::Command { command } => {
                    if app.apply_session_command(command, ActionSource::Remote) {
                        diagnostics::log_info(
                            "distributed",
                            format!("applied session command: {:?}", command),
                        );
                        self.revision = self.revision.saturating_add(1);
                        self.broadcast_message(HostMessage::Ack {
                            revision: self.revision,
                            command,
                        });
                    }
                }
                ReceivedClientMessage::UiIntent { client_id, intent } => {
                    if self.apply_remote_ui_intent(app, client_id, intent) {
                        diagnostics::log_info(
                            "distributed",
                            format!("applied ui intent for client {client_id}"),
                        );
                        self.revision = self.revision.saturating_add(1);
                    }
                }
            }
        }

        if self.last_snapshot_broadcast_at.elapsed() >= SNAPSHOT_BROADCAST_INTERVAL {
            let snapshot = app.session_snapshot(self.revision, self.connected_clients());
            if let Ok(snapshot_json) = serde_json::to_string(&snapshot) {
                if self.last_snapshot_json.as_ref() != Some(&snapshot_json) {
                    self.revision = self.revision.saturating_add(1);
                    let snapshot = app.session_snapshot(self.revision, self.connected_clients());
                    self.broadcast_message(HostMessage::Snapshot { snapshot });
                    self.last_snapshot_json = Some(snapshot_json);
                } else {
                    self.broadcast_message(HostMessage::Snapshot { snapshot });
                }
            }
            self.last_snapshot_broadcast_at = Instant::now();
        }
    }

    pub fn connected_clients(&self) -> usize {
        self.connected_client_count.load(Ordering::Relaxed)
    }

    fn apply_remote_ui_intent(
        &mut self,
        app: &mut App,
        client_id: usize,
        intent: RemoteUiIntent,
    ) -> bool {
        let context = self
            .contexts
            .entry(client_id)
            .or_insert_with(|| ClientContext {
                client_name: format!("client-{client_id}"),
                ui_state: ClientUiState::default(),
            });
        let host_ui_state = app.capture_client_ui_state();
        let before_snapshot = serde_json::to_string(&app.session_snapshot(0, 0)).ok();
        app.apply_client_ui_state(&context.ui_state);
        let control = app.apply_remote_ui_intent(intent);
        context.ui_state = app.capture_client_ui_state();
        app.apply_client_ui_state(&host_ui_state);

        match control.unwrap_or(AppControl::Continue) {
            AppControl::Quit => false,
            AppControl::Continue => {
                let after_snapshot = serde_json::to_string(&app.session_snapshot(0, 0)).ok();
                before_snapshot != after_snapshot
            }
        }
    }

    fn broadcast_message(&self, message: HostMessage) {
        let Ok(mut guard) = self.clients.lock() else {
            return;
        };
        guard.retain(|_, client| client.send(message.clone()).is_ok());
    }
}

fn spawn_client_writer(stream: TcpStream, outbound_rx: Receiver<HostMessage>) {
    thread::spawn(move || {
        let mut writer = stream;
        for message in outbound_rx {
            let Ok(line) = serde_json::to_string(&message) else {
                continue;
            };
            if writeln!(writer, "{line}").is_err() {
                break;
            }
            if writer.flush().is_err() {
                break;
            }
        }
    });
}

fn spawn_client_reader(
    stream: TcpStream,
    client_id: usize,
    command_tx: Sender<ReceivedClientMessage>,
) {
    thread::spawn(move || {
        let reader = BufReader::new(stream);
        for line in reader.lines() {
            let Ok(line) = line else {
                break;
            };
            let Ok(message) = serde_json::from_str::<ThinClientMessage>(&line) else {
                continue;
            };
            match message {
                ThinClientMessage::Hello {
                    client_name: hello_name,
                } => {
                    let _ = command_tx.send(ReceivedClientMessage::Hello {
                        client_id,
                        client_name: hello_name,
                    });
                }
                ThinClientMessage::Command { command } => {
                    if command_tx
                        .send(ReceivedClientMessage::Command { command })
                        .is_err()
                    {
                        break;
                    }
                }
                ThinClientMessage::UiIntent { intent } => {
                    if command_tx
                        .send(ReceivedClientMessage::UiIntent { client_id, intent })
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
        let _ = command_tx.send(ReceivedClientMessage::Disconnected { client_id });
    });
}

pub fn run_headless_session_host(
    mut app: App,
    listen_addr: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut server = SessionServer::bind(
        listen_addr,
        DiscoveryHostConfig {
            session_name: default_discovery_session_name(&app.session_project().name),
            host_mode: "host-session".to_string(),
            listen_addr: listen_addr.to_string(),
        },
    )?;
    let (stdin_tx, stdin_rx) = mpsc::channel::<String>();
    spawn_stdin_reader(stdin_tx);

    println!("trekr headless session host listening on {listen_addr}");
    diagnostics::log_info(
        "distributed",
        format!(
            "headless session host listening on {listen_addr}; log file {}",
            diagnostics::log_path().display()
        ),
    );
    println!(
        "Local commands: {}",
        SessionCommand::ALL
            .map(|command| command.label())
            .join(", ")
    );
    println!("Type `quit` to stop the host.");

    let mut last_tick_at = Instant::now();
    let mut last_status_at = Instant::now() - HEADLESS_STATUS_INTERVAL;
    loop {
        for line in stdin_rx.try_iter() {
            let trimmed = line.trim();
            if trimmed.eq_ignore_ascii_case("quit") || trimmed.eq_ignore_ascii_case("exit") {
                return Ok(());
            }
            if let Some(command) = SessionCommand::parse_token(trimmed) {
                let _ = app.apply_session_command(command, ActionSource::Keyboard);
            } else if !trimmed.is_empty() {
                println!("Unknown command: {trimmed}");
            }
        }

        let now = Instant::now();
        app.service_session_runtime(now.saturating_duration_since(last_tick_at));
        last_tick_at = now;
        server.service_app(&mut app);

        if last_status_at.elapsed() >= HEADLESS_STATUS_INTERVAL {
            println!(
                "host | tick={} playhead={} playing={} recording={} active_track={} clients={}",
                app.session_transport_ticks(),
                app.session_playhead_ticks(),
                app.session_project().transport.playing,
                app.session_project().transport.recording,
                app.session_project().active_track_index + 1,
                server.connected_clients(),
            );
            last_status_at = Instant::now();
        }

        thread::sleep(IO_POLL_INTERVAL);
    }
}

pub fn run_thin_client(
    connect_addr: &str,
    client_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut writer, snapshot_rx) = connect_thin_client_channel(connect_addr, client_name)?;

    let (stdin_tx, stdin_rx) = mpsc::channel::<String>();
    spawn_stdin_reader(stdin_tx);

    let mut latest_snapshot: Option<SessionSnapshot> = None;
    let mut status_line = format!("Connected to {connect_addr} as {client_name}");
    let mut last_render_at = Instant::now() - Duration::from_millis(250);
    loop {
        for message in snapshot_rx.try_iter() {
            match message {
                HostMessage::Snapshot { snapshot } => latest_snapshot = Some(snapshot),
                HostMessage::Ack { revision, command } => {
                    status_line = format!("Ack rev {revision} for {}", command.label())
                }
                HostMessage::Reject { message } => status_line = format!("Rejected: {message}"),
            }
        }

        for line in stdin_rx.try_iter() {
            let trimmed = line.trim();
            if trimmed.eq_ignore_ascii_case("quit") || trimmed.eq_ignore_ascii_case("exit") {
                return Ok(());
            }
            if let Some(command) = SessionCommand::parse_token(trimmed) {
                writeln!(
                    writer,
                    "{}",
                    serde_json::to_string(&ThinClientMessage::Command { command })?
                )?;
                writer.flush()?;
                status_line = format!("Sent {}", command.label());
            } else if !trimmed.is_empty() {
                status_line = format!("Unknown command: {trimmed}");
            }
        }

        if last_render_at.elapsed() >= Duration::from_millis(250) {
            print!("\x1B[2J\x1B[1;1H");
            println!("trekr thin client | host {connect_addr} | client {client_name}");
            println!(
                "commands: {}",
                SessionCommand::ALL
                    .map(|command| command.label())
                    .join(", ")
            );
            println!("type quit to exit");
            println!();
            if let Some(snapshot) = &latest_snapshot {
                let active = snapshot
                    .project
                    .tracks
                    .get(snapshot.project.active_track_index)
                    .map(|track| track.name.as_str())
                    .unwrap_or("<missing>");
                println!("revision: {}", snapshot.revision);
                println!("connected clients: {}", snapshot.connected_clients);
                println!(
                    "transport: playing={} recording={} loop={} tempo={}bpm",
                    snapshot.project.transport.playing,
                    snapshot.project.transport.recording,
                    snapshot.project.transport.loop_enabled,
                    snapshot.project.transport.tempo_bpm
                );
                println!(
                    "ticks: transport={} playhead={}",
                    snapshot.transport_ticks, snapshot.playhead_ticks
                );
                println!(
                    "active track: {} ({}) arm={} mute={} solo={} passthrough={}",
                    snapshot.project.active_track_index + 1,
                    active,
                    snapshot.project.tracks[snapshot.project.active_track_index]
                        .state
                        .armed,
                    snapshot.project.tracks[snapshot.project.active_track_index]
                        .state
                        .muted,
                    snapshot.project.tracks[snapshot.project.active_track_index]
                        .state
                        .soloed,
                    snapshot.project.tracks[snapshot.project.active_track_index]
                        .state
                        .passthrough,
                );
            } else {
                println!("Waiting for first snapshot...");
            }
            println!();
            println!("status: {status_line}");
            io::stdout().flush()?;
            last_render_at = Instant::now();
        }

        thread::sleep(IO_POLL_INTERVAL);
    }
}

#[derive(Debug, Clone)]
enum ThinClientShellMode {
    Discovery,
    ManualAddress,
    Connecting {
        connect_addr: String,
    },
    ConnectFailed {
        connect_addr: String,
        message: String,
    },
}

#[derive(Debug, Clone)]
struct ThinClientShellState {
    mode: ThinClientShellMode,
    discovery_enabled: bool,
    sessions: Vec<DiscoveredSession>,
    selected_index: usize,
    manual_address: String,
    status_line: String,
}

impl ThinClientShellState {
    fn new(initial_connect_addr: Option<&str>) -> Self {
        let manual_address = initial_connect_addr.unwrap_or_default().to_string();
        let mode = initial_connect_addr
            .map(|connect_addr| ThinClientShellMode::Connecting {
                connect_addr: connect_addr.to_string(),
            })
            .unwrap_or(ThinClientShellMode::Discovery);
        Self {
            mode,
            discovery_enabled: initial_connect_addr.is_none(),
            sessions: Vec::new(),
            selected_index: 0,
            manual_address,
            status_line: if initial_connect_addr.is_some() {
                "Connecting...".to_string()
            } else {
                format!("Searching for trekr hosts on UDP {DISCOVERY_UDP_PORT}")
            },
        }
    }

    fn selected_session(&self) -> Option<&DiscoveredSession> {
        self.sessions.get(self.selected_index)
    }

    fn refresh_selection_bounds(&mut self) {
        if self.sessions.is_empty() {
            self.selected_index = 0;
        } else {
            self.selected_index = self
                .selected_index
                .min(self.sessions.len().saturating_sub(1));
        }
    }
}

enum ThinClientShellOutcome {
    Continue,
    Quit,
    Connect(String),
}

pub fn run_thin_client_sdl(
    connect_addr: Option<&str>,
    client_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let video = sdl_context.video()?;
    let window = video
        .window(&format!("trekr thin client - {client_name}"), 960, 540)
        .position_centered()
        .resizable()
        .high_pixel_density()
        .build()
        .map_err(|err| err.to_string())?;
    let mut canvas = window.into_canvas();
    let mut event_pump = sdl_context.event_pump()?;
    let mut discovery_client = DiscoveryClient::new(client_name).ok();
    let mut shell = ThinClientShellState::new(connect_addr);
    let active_theme = theme(ThemePreset::DefaultDark);

    if connect_addr.is_none() && discovery_client.is_none() {
        shell.status_line =
            format!("LAN discovery unavailable on this device. Use Manual Connect or --connect.");
    }

    loop {
        if shell.discovery_enabled {
            if let Some(discovery_client) = &mut discovery_client {
                discovery_client.poll(&mut shell.sessions);
                shell.refresh_selection_bounds();
            }
        }

        for event in event_pump.poll_iter() {
            let converted = event.get_converted_coords(&canvas).unwrap_or(event.clone());
            match handle_shell_event(&mut shell, &mut discovery_client, &converted, &canvas) {
                ThinClientShellOutcome::Continue => {}
                ThinClientShellOutcome::Quit => return Ok(()),
                ThinClientShellOutcome::Connect(addr) => {
                    shell.mode = ThinClientShellMode::Connecting {
                        connect_addr: addr.clone(),
                    };
                    shell.manual_address = addr;
                    shell.status_line = "Connecting...".to_string();
                }
            }
        }

        if let ThinClientShellMode::Connecting { connect_addr } = shell.mode.clone() {
            diagnostics::log_info(
                "distributed",
                format!("thin client {client_name} connecting to {connect_addr}"),
            );
            match connect_thin_client_channel(&connect_addr, client_name) {
                Ok((writer, snapshot_rx)) => {
                    diagnostics::log_info(
                        "distributed",
                        format!("thin client {client_name} connected to {connect_addr}"),
                    );
                    return run_connected_thin_client_sdl(
                        &mut canvas,
                        &mut event_pump,
                        client_name,
                        &connect_addr,
                        writer,
                        snapshot_rx,
                    );
                }
                Err(err) => {
                    diagnostics::log_error(
                        "distributed",
                        format!(
                            "thin client {client_name} failed to connect to {connect_addr}: {err}"
                        ),
                    );
                    shell.status_line = format!("Connection failed: {err}");
                    shell.mode = ThinClientShellMode::ConnectFailed {
                        connect_addr: connect_addr.clone(),
                        message: err.to_string(),
                    };
                    shell.manual_address = connect_addr;
                }
            }
        }

        draw_thin_client_shell(&mut canvas, active_theme, client_name, &shell)?;
        thread::sleep(SDL_CLIENT_FRAME_INTERVAL);
    }
}

fn run_connected_thin_client_sdl(
    canvas: &mut Canvas<Window>,
    event_pump: &mut sdl3::EventPump,
    client_name: &str,
    connect_addr: &str,
    mut writer: TcpStream,
    snapshot_rx: Receiver<HostMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
    let active_theme = theme(ThemePreset::DefaultDark);
    let mut mirror_app = App::new_demo();
    let mut latest_snapshot: Option<SessionSnapshot> = None;
    let mut status_line = format!("Connected to {connect_addr} as {client_name}");

    'running: loop {
        mirror_app.configure_window_canvas(canvas)?;
        for event in event_pump.poll_iter() {
            match &event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    repeat: false,
                    ..
                } => break 'running,
                _ => {
                    let converted = event.get_converted_coords(canvas).unwrap_or(event.clone());
                    match &converted {
                        Event::KeyDown {
                            keycode: Some(keycode),
                            keymod,
                            repeat,
                            ..
                        } => {
                            if let Some(intent) =
                                mirror_app.resolve_remote_key_intent(*keycode, *keymod, *repeat)
                            {
                                send_thin_client_ui_intent(&mut writer, intent.clone())?;
                                let _ = mirror_app.apply_remote_ui_intent(intent);
                                status_line = "Sent remote key intent".to_string();
                            }
                        }
                        Event::MouseMotion { x, y, .. } | Event::FingerMotion { x, y, .. } => {
                            let _ = mirror_app.handle_remote_pointer_hover(*x as i32, *y as i32);
                        }
                        Event::MouseButtonDown { x, y, .. } => {
                            if let Some(intent) = mirror_app.resolve_remote_pointer_intent(
                                *x as i32,
                                *y as i32,
                                ActionSource::Pointer,
                            ) {
                                send_thin_client_ui_intent(&mut writer, intent.clone())?;
                                let _ = mirror_app.apply_remote_ui_intent(intent);
                                status_line = "Sent remote pointer intent".to_string();
                            }
                        }
                        Event::FingerDown { x, y, .. } => {
                            if let Some(intent) = mirror_app.resolve_remote_pointer_intent(
                                *x as i32,
                                *y as i32,
                                ActionSource::Touch,
                            ) {
                                send_thin_client_ui_intent(&mut writer, intent.clone())?;
                                let _ = mirror_app.apply_remote_ui_intent(intent);
                                status_line = "Sent remote touch intent".to_string();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        for message in snapshot_rx.try_iter() {
            match message {
                HostMessage::Snapshot { snapshot } => {
                    mirror_app.apply_session_snapshot(&snapshot);
                    latest_snapshot = Some(snapshot);
                }
                HostMessage::Ack { revision, command } => {
                    status_line = format!("Ack rev {revision} for {}", command.label())
                }
                HostMessage::Reject { message } => status_line = format!("Rejected: {message}"),
            }
        }

        canvas.window_mut().set_title(&format!(
            "trekr thin client - {client_name} | {status_line}"
        ))?;
        if latest_snapshot.is_some() {
            mirror_app.configure_window_canvas(canvas)?;
            mirror_app.draw_window(canvas)?;
        } else {
            draw_waiting_thin_client(canvas, active_theme, connect_addr, client_name)?;
        }
        thread::sleep(SDL_CLIENT_FRAME_INTERVAL);
    }

    Ok(())
}

fn connect_thin_client_channel(
    connect_addr: &str,
    client_name: &str,
) -> Result<(TcpStream, Receiver<HostMessage>), Box<dyn std::error::Error>> {
    let stream = TcpStream::connect(connect_addr)?;
    let mut writer = stream.try_clone()?;
    let reader_stream = stream.try_clone()?;
    writeln!(
        writer,
        "{}",
        serde_json::to_string(&ThinClientMessage::Hello {
            client_name: client_name.to_string(),
        })?
    )?;
    writer.flush()?;

    let (snapshot_tx, snapshot_rx) = mpsc::channel::<HostMessage>();
    thread::spawn(move || {
        let reader = BufReader::new(reader_stream);
        for line in reader.lines() {
            let Ok(line) = line else {
                break;
            };
            let Ok(message) = serde_json::from_str::<HostMessage>(&line) else {
                continue;
            };
            if snapshot_tx.send(message).is_err() {
                break;
            }
        }
    });
    Ok((writer, snapshot_rx))
}

fn handle_shell_event(
    shell: &mut ThinClientShellState,
    discovery_client: &mut Option<DiscoveryClient>,
    event: &Event,
    canvas: &Canvas<Window>,
) -> ThinClientShellOutcome {
    match event {
        Event::Quit { .. } => ThinClientShellOutcome::Quit,
        Event::KeyDown {
            keycode: Some(keycode),
            repeat: false,
            ..
        } => handle_shell_key_down(shell, discovery_client, *keycode),
        Event::TextInput { text, .. } => {
            if matches!(shell.mode, ThinClientShellMode::ManualAddress) {
                shell.manual_address.push_str(text);
            }
            ThinClientShellOutcome::Continue
        }
        Event::MouseButtonDown { x, y, .. } | Event::FingerDown { x, y, .. } => {
            handle_shell_pointer_down(shell, discovery_client, *x as i32, *y as i32, canvas)
        }
        _ => ThinClientShellOutcome::Continue,
    }
}

fn handle_shell_key_down(
    shell: &mut ThinClientShellState,
    discovery_client: &mut Option<DiscoveryClient>,
    keycode: Keycode,
) -> ThinClientShellOutcome {
    match &mut shell.mode {
        ThinClientShellMode::Discovery => match keycode {
            Keycode::Escape => ThinClientShellOutcome::Quit,
            Keycode::Down => {
                if !shell.sessions.is_empty() {
                    shell.selected_index =
                        (shell.selected_index + 1).min(shell.sessions.len().saturating_sub(1));
                }
                ThinClientShellOutcome::Continue
            }
            Keycode::Up => {
                shell.selected_index = shell.selected_index.saturating_sub(1);
                ThinClientShellOutcome::Continue
            }
            Keycode::Return => {
                if let Some(session) = shell.selected_session() {
                    if session.is_compatible() {
                        ThinClientShellOutcome::Connect(session.preferred_connect_addr.clone())
                    } else {
                        shell.status_line =
                            "Selected session uses an incompatible discovery protocol.".to_string();
                        ThinClientShellOutcome::Continue
                    }
                } else {
                    ThinClientShellOutcome::Continue
                }
            }
            Keycode::R => {
                if let Some(discovery_client) = discovery_client {
                    discovery_client.request_refresh();
                }
                shell.status_line = "Refreshing LAN session discovery...".to_string();
                ThinClientShellOutcome::Continue
            }
            Keycode::M => {
                shell.mode = ThinClientShellMode::ManualAddress;
                ThinClientShellOutcome::Continue
            }
            _ => ThinClientShellOutcome::Continue,
        },
        ThinClientShellMode::ManualAddress => match keycode {
            Keycode::Escape => {
                shell.discovery_enabled = true;
                shell.mode = ThinClientShellMode::Discovery;
                ThinClientShellOutcome::Continue
            }
            Keycode::Backspace => {
                shell.manual_address.pop();
                ThinClientShellOutcome::Continue
            }
            Keycode::Return => {
                let address = shell.manual_address.trim().to_string();
                if address.is_empty() {
                    shell.status_line = "Enter a host:port address first.".to_string();
                    ThinClientShellOutcome::Continue
                } else {
                    ThinClientShellOutcome::Connect(address)
                }
            }
            _ => {
                if let Some(text) = shell_text_for_keycode(keycode) {
                    shell.manual_address.push_str(text);
                }
                ThinClientShellOutcome::Continue
            }
        },
        ThinClientShellMode::ConnectFailed { connect_addr, .. } => match keycode {
            Keycode::Escape => {
                shell.discovery_enabled = true;
                shell.mode = ThinClientShellMode::Discovery;
                ThinClientShellOutcome::Continue
            }
            Keycode::Return | Keycode::R => ThinClientShellOutcome::Connect(connect_addr.clone()),
            Keycode::M => {
                shell.manual_address = connect_addr.clone();
                shell.mode = ThinClientShellMode::ManualAddress;
                ThinClientShellOutcome::Continue
            }
            _ => ThinClientShellOutcome::Continue,
        },
        ThinClientShellMode::Connecting { .. } => ThinClientShellOutcome::Continue,
    }
}

fn handle_shell_pointer_down(
    shell: &mut ThinClientShellState,
    discovery_client: &mut Option<DiscoveryClient>,
    x: i32,
    y: i32,
    canvas: &Canvas<Window>,
) -> ThinClientShellOutcome {
    let layout = discovery_screen_layout(canvas);
    if rect_contains(layout.refresh_button, x, y) {
        if let Some(discovery_client) = discovery_client {
            discovery_client.request_refresh();
        }
        shell.status_line = "Refreshing LAN session discovery...".to_string();
        return ThinClientShellOutcome::Continue;
    }
    if rect_contains(layout.manual_button, x, y) {
        shell.mode = ThinClientShellMode::ManualAddress;
        return ThinClientShellOutcome::Continue;
    }

    match shell.mode {
        ThinClientShellMode::Discovery => {
            for (index, row) in layout.session_rows.iter().enumerate() {
                if rect_contains(*row, x, y) {
                    shell.selected_index = index.min(shell.sessions.len().saturating_sub(1));
                    return ThinClientShellOutcome::Continue;
                }
            }
            if rect_contains(layout.connect_button, x, y) {
                if let Some(session) = shell.selected_session() {
                    if session.is_compatible() {
                        return ThinClientShellOutcome::Connect(
                            session.preferred_connect_addr.clone(),
                        );
                    }
                    shell.status_line =
                        "Selected session uses an incompatible discovery protocol.".to_string();
                }
                return ThinClientShellOutcome::Continue;
            }
        }
        ThinClientShellMode::ManualAddress => {
            if rect_contains(layout.back_button, x, y) {
                shell.discovery_enabled = true;
                shell.mode = ThinClientShellMode::Discovery;
                return ThinClientShellOutcome::Continue;
            }
            if rect_contains(layout.connect_button, x, y) {
                let address = shell.manual_address.trim().to_string();
                if address.is_empty() {
                    shell.status_line = "Enter a host:port address first.".to_string();
                    return ThinClientShellOutcome::Continue;
                }
                return ThinClientShellOutcome::Connect(address);
            }
        }
        ThinClientShellMode::ConnectFailed {
            ref connect_addr, ..
        } => {
            if rect_contains(layout.back_button, x, y) {
                shell.discovery_enabled = true;
                shell.mode = ThinClientShellMode::Discovery;
                return ThinClientShellOutcome::Continue;
            }
            if rect_contains(layout.connect_button, x, y) {
                return ThinClientShellOutcome::Connect(connect_addr.clone());
            }
        }
        ThinClientShellMode::Connecting { .. } => {}
    }

    ThinClientShellOutcome::Continue
}

struct DiscoveryScreenLayout {
    outer: Rect,
    session_rows: Vec<Rect>,
    refresh_button: Rect,
    manual_button: Rect,
    connect_button: Rect,
    back_button: Rect,
    manual_input: Rect,
}

fn discovery_screen_layout(canvas: &Canvas<Window>) -> DiscoveryScreenLayout {
    let (width, height) = canvas.output_size().unwrap_or((960, 540));
    let outer = Rect::new(12, 12, width.saturating_sub(24), height.saturating_sub(24));
    let session_rows = (0..5)
        .map(|index| {
            Rect::new(
                outer.x + 12,
                outer.y + 90 + (index as i32 * 54),
                outer.width().saturating_sub(24),
                44,
            )
        })
        .collect();
    DiscoveryScreenLayout {
        outer,
        session_rows,
        refresh_button: Rect::new(outer.x + 12, outer.y + 44, 96, 24),
        manual_button: Rect::new(outer.x + 116, outer.y + 44, 132, 24),
        connect_button: Rect::new(
            outer.x + outer.width() as i32 - 132,
            outer.y + outer.height() as i32 - 36,
            120,
            24,
        ),
        back_button: Rect::new(outer.x + 12, outer.y + outer.height() as i32 - 36, 96, 24),
        manual_input: Rect::new(
            outer.x + 12,
            outer.y + 96,
            outer.width().saturating_sub(24),
            28,
        ),
    }
}

fn draw_thin_client_shell(
    canvas: &mut Canvas<Window>,
    active_theme: &crate::theme::Theme,
    client_name: &str,
    shell: &ThinClientShellState,
) -> Result<(), Box<dyn std::error::Error>> {
    let layout = discovery_screen_layout(canvas);
    canvas.set_draw_color(active_theme.app_chrome.window_clear);
    canvas.clear();
    canvas.set_draw_color(active_theme.app_chrome.surface_fill);
    canvas.fill_rect(layout.outer)?;
    canvas.set_draw_color(active_theme.app_chrome.surface_border);
    canvas.draw_rect(layout.outer)?;

    crate::ui::draw_text(
        canvas,
        "CONNECT",
        layout.outer.x + 12,
        layout.outer.y + 12,
        2,
        active_theme.app_chrome.tab_text_active,
    )?;
    crate::ui::draw_text_fitted(
        canvas,
        &format!("client {client_name}"),
        Rect::new(
            layout.outer.x + 12,
            layout.outer.y + 28,
            layout.outer.width().saturating_sub(24),
            8,
        ),
        1,
        active_theme.app_chrome.detail_text,
    )?;

    draw_shell_button(canvas, active_theme, layout.refresh_button, "Refresh", true)?;
    draw_shell_button(canvas, active_theme, layout.manual_button, "Manual", true)?;

    match &shell.mode {
        ThinClientShellMode::Discovery => {
            crate::ui::draw_text_fitted(
                canvas,
                "Discovered sessions",
                Rect::new(layout.outer.x + 12, layout.outer.y + 72, 200, 8),
                1,
                active_theme.app_chrome.detail_text,
            )?;
            if shell.sessions.is_empty() {
                crate::ui::draw_text_fitted(
                    canvas,
                    "Searching for trekr sessions on the local network...",
                    Rect::new(
                        layout.outer.x + 12,
                        layout.outer.y + 102,
                        layout.outer.width().saturating_sub(24),
                        8,
                    ),
                    1,
                    active_theme.app_chrome.detail_text,
                )?;
            } else {
                for (index, row) in layout.session_rows.iter().enumerate() {
                    let Some(session) = shell.sessions.get(index) else {
                        continue;
                    };
                    let selected = index == shell.selected_index;
                    let row_fill = if selected {
                        active_theme.app_chrome.tab_active_fill
                    } else {
                        active_theme.app_chrome.tab_inactive_fill
                    };
                    canvas.set_draw_color(row_fill);
                    canvas.fill_rect(*row)?;
                    canvas.set_draw_color(active_theme.app_chrome.surface_border);
                    canvas.draw_rect(*row)?;
                    crate::ui::draw_text_fitted(
                        canvas,
                        &session.session.session_name,
                        Rect::new(row.x + 8, row.y + 6, row.width().saturating_sub(16), 8),
                        1,
                        active_theme.app_chrome.tab_text_active,
                    )?;
                    let subtitle = if session.is_compatible() {
                        session.subtitle()
                    } else {
                        format!(
                            "{} | incompatible protocol v{}",
                            session.preferred_connect_addr, session.session.protocol_version
                        )
                    };
                    crate::ui::draw_text_fitted(
                        canvas,
                        &subtitle,
                        Rect::new(row.x + 8, row.y + 20, row.width().saturating_sub(16), 8),
                        1,
                        active_theme.app_chrome.detail_text,
                    )?;
                }
            }
            draw_shell_button(
                canvas,
                active_theme,
                layout.connect_button,
                "Connect",
                shell
                    .selected_session()
                    .is_some_and(DiscoveredSession::is_compatible),
            )?;
        }
        ThinClientShellMode::ManualAddress => {
            crate::ui::draw_text_fitted(
                canvas,
                "Manual host address",
                Rect::new(layout.outer.x + 12, layout.outer.y + 72, 220, 8),
                1,
                active_theme.app_chrome.detail_text,
            )?;
            canvas.set_draw_color(active_theme.app_chrome.tab_inactive_fill);
            canvas.fill_rect(layout.manual_input)?;
            canvas.set_draw_color(active_theme.app_chrome.surface_border);
            canvas.draw_rect(layout.manual_input)?;
            crate::ui::draw_text_fitted(
                canvas,
                if shell.manual_address.is_empty() {
                    "host:port"
                } else {
                    &shell.manual_address
                },
                Rect::new(
                    layout.manual_input.x + 8,
                    layout.manual_input.y + 8,
                    layout.manual_input.width().saturating_sub(16),
                    8,
                ),
                1,
                active_theme.app_chrome.tab_text_active,
            )?;
            draw_shell_button(canvas, active_theme, layout.back_button, "Back", true)?;
            draw_shell_button(
                canvas,
                active_theme,
                layout.connect_button,
                "Connect",
                !shell.manual_address.trim().is_empty(),
            )?;
        }
        ThinClientShellMode::Connecting { connect_addr } => {
            crate::ui::draw_text_fitted(
                canvas,
                &format!("Connecting to {connect_addr}..."),
                Rect::new(
                    layout.outer.x + 12,
                    layout.outer.y + 104,
                    layout.outer.width().saturating_sub(24),
                    8,
                ),
                1,
                active_theme.app_chrome.detail_text,
            )?;
        }
        ThinClientShellMode::ConnectFailed {
            connect_addr,
            message,
        } => {
            crate::ui::draw_text_fitted(
                canvas,
                &format!("Failed to connect to {connect_addr}"),
                Rect::new(
                    layout.outer.x + 12,
                    layout.outer.y + 88,
                    layout.outer.width().saturating_sub(24),
                    8,
                ),
                1,
                active_theme.app_chrome.detail_text,
            )?;
            crate::ui::draw_text_fitted(
                canvas,
                message,
                Rect::new(
                    layout.outer.x + 12,
                    layout.outer.y + 104,
                    layout.outer.width().saturating_sub(24),
                    24,
                ),
                1,
                active_theme.app_chrome.detail_text,
            )?;
            draw_shell_button(canvas, active_theme, layout.back_button, "Back", true)?;
            draw_shell_button(canvas, active_theme, layout.connect_button, "Retry", true)?;
        }
    }

    crate::ui::draw_text_fitted(
        canvas,
        &shell.status_line,
        Rect::new(
            layout.outer.x + 12,
            layout.outer.y + layout.outer.height() as i32 - 56,
            layout.outer.width().saturating_sub(24),
            8,
        ),
        1,
        active_theme.app_chrome.detail_text,
    )?;
    canvas.present();
    Ok(())
}

fn draw_shell_button(
    canvas: &mut Canvas<Window>,
    active_theme: &crate::theme::Theme,
    rect: Rect,
    label: &str,
    enabled: bool,
) -> Result<(), String> {
    let fill = if enabled {
        active_theme.app_chrome.tab_active_fill
    } else {
        active_theme.app_chrome.tab_inactive_fill
    };
    let text = if enabled {
        active_theme.app_chrome.tab_text_active
    } else {
        active_theme.app_chrome.detail_text
    };
    canvas.set_draw_color(fill);
    canvas.fill_rect(rect).map_err(|err| err.to_string())?;
    canvas.set_draw_color(active_theme.app_chrome.surface_border);
    canvas.draw_rect(rect).map_err(|err| err.to_string())?;
    crate::ui::draw_text_fitted(
        canvas,
        label,
        Rect::new(rect.x + 8, rect.y + 8, rect.width().saturating_sub(16), 8),
        1,
        text,
    )
}

fn shell_text_for_keycode(keycode: Keycode) -> Option<&'static str> {
    Some(match keycode {
        Keycode::Colon => ":",
        Keycode::Period => ".",
        Keycode::Minus => "-",
        Keycode::Slash => "/",
        Keycode::_0 | Keycode::Kp0 => "0",
        Keycode::_1 | Keycode::Kp1 => "1",
        Keycode::_2 | Keycode::Kp2 => "2",
        Keycode::_3 | Keycode::Kp3 => "3",
        Keycode::_4 | Keycode::Kp4 => "4",
        Keycode::_5 | Keycode::Kp5 => "5",
        Keycode::_6 | Keycode::Kp6 => "6",
        Keycode::_7 | Keycode::Kp7 => "7",
        Keycode::_8 | Keycode::Kp8 => "8",
        Keycode::_9 | Keycode::Kp9 => "9",
        Keycode::A => "a",
        Keycode::B => "b",
        Keycode::C => "c",
        Keycode::D => "d",
        Keycode::E => "e",
        Keycode::F => "f",
        Keycode::G => "g",
        Keycode::H => "h",
        Keycode::I => "i",
        Keycode::J => "j",
        Keycode::K => "k",
        Keycode::L => "l",
        Keycode::M => "m",
        Keycode::N => "n",
        Keycode::O => "o",
        Keycode::P => "p",
        Keycode::Q => "q",
        Keycode::R => "r",
        Keycode::S => "s",
        Keycode::T => "t",
        Keycode::U => "u",
        Keycode::V => "v",
        Keycode::W => "w",
        Keycode::X => "x",
        Keycode::Y => "y",
        Keycode::Z => "z",
        _ => return None,
    })
}

fn send_thin_client_ui_intent(
    writer: &mut TcpStream,
    intent: RemoteUiIntent,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(
        writer,
        "{}",
        serde_json::to_string(&ThinClientMessage::UiIntent { intent })?
    )?;
    writer.flush()?;
    Ok(())
}

pub fn default_discovery_session_name(project_name: &str) -> String {
    let host_name = local_host_name();
    let trimmed = project_name.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("Untitled") {
        format!("trekr on {host_name}")
    } else {
        format!("{trimmed} on {host_name}")
    }
}

fn local_host_name() -> String {
    env::var("COMPUTERNAME")
        .or_else(|_| env::var("HOSTNAME"))
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "localhost".to_string())
}

fn discover_connect_addrs(listen_addr: &str, host_name: &str) -> Vec<String> {
    let port = parse_listen_port(listen_addr);
    let bind_host = parse_listen_host(listen_addr);
    let mut candidates = Vec::new();

    match bind_host.as_deref() {
        Some("127.0.0.1") | Some("localhost") => {
            candidates.push(format!("127.0.0.1:{port}"));
        }
        Some(host) if !host.is_empty() && host != "0.0.0.0" => {
            candidates.push(format!("{host}:{port}"));
        }
        _ => {
            if let Ok(ifaces) = get_if_addrs() {
                for iface in ifaces {
                    let IfAddr::V4(addr) = iface.addr else {
                        continue;
                    };
                    let ip = addr.ip;
                    if ip.is_unspecified() {
                        continue;
                    }
                    candidates.push(format!("{ip}:{port}"));
                }
            }
            candidates.push(format!("{host_name}:{port}"));
        }
    }

    candidates.push(format!("127.0.0.1:{port}"));
    dedupe_preserve_order(candidates)
}

fn choose_preferred_connect_addr(
    session: &DiscoveryAdvertisement,
    responder_ip: String,
) -> Option<String> {
    let local_host = local_host_name();
    let prefer_loopback = session.host_name.eq_ignore_ascii_case(&local_host);
    let mut candidates = session.connect_addrs.clone();
    candidates.push(format!("{responder_ip}:{}", session.port));
    let candidates = dedupe_preserve_order(candidates);

    let mut scored = candidates
        .into_iter()
        .map(|addr| {
            let latency = probe_connect_latency(&addr);
            let score = connect_addr_score(&addr, prefer_loopback, latency);
            (addr, score, latency)
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| {
        left.1
            .cmp(&right.1)
            .then_with(|| left.2.cmp(&right.2))
            .then_with(|| left.0.cmp(&right.0))
    });
    scored.into_iter().map(|entry| entry.0).next()
}

fn probe_connect_latency(connect_addr: &str) -> Duration {
    let start = Instant::now();
    let result = connect_addr
        .parse::<SocketAddr>()
        .ok()
        .map(|addr| TcpStream::connect_timeout(&addr, CONNECT_PROBE_TIMEOUT));
    if let Some(Ok(stream)) = result {
        let _ = stream.shutdown(std::net::Shutdown::Both);
        start.elapsed()
    } else {
        CONNECT_PROBE_TIMEOUT + Duration::from_millis(500)
    }
}

fn connect_addr_score(connect_addr: &str, prefer_loopback: bool, latency: Duration) -> (u8, u128) {
    let host = connect_addr
        .rsplit_once(':')
        .map(|(host, _)| host.trim_matches(['[', ']']))
        .unwrap_or(connect_addr);

    let class = if host == "127.0.0.1" || host.eq_ignore_ascii_case("localhost") {
        if prefer_loopback { 0 } else { 3 }
    } else if let Ok(ip) = host.parse::<std::net::Ipv4Addr>() {
        if is_private_v4(ip) {
            1
        } else if is_link_local_v4(ip) {
            4
        } else {
            5
        }
    } else {
        2
    };
    (class, latency.as_millis())
}

fn is_private_v4(ip: std::net::Ipv4Addr) -> bool {
    let octets = ip.octets();
    octets[0] == 10
        || (octets[0] == 172 && (16..=31).contains(&octets[1]))
        || (octets[0] == 192 && octets[1] == 168)
}

fn is_link_local_v4(ip: std::net::Ipv4Addr) -> bool {
    let octets = ip.octets();
    octets[0] == 169 && octets[1] == 254
}

fn parse_listen_host(listen_addr: &str) -> Option<String> {
    listen_addr
        .rsplit_once(':')
        .map(|(host, _)| host.to_string())
}

fn parse_listen_port(listen_addr: &str) -> u16 {
    listen_addr
        .rsplit_once(':')
        .and_then(|(_, port)| port.parse::<u16>().ok())
        .unwrap_or(8788)
}

fn dedupe_preserve_order(values: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut deduped = Vec::new();
    for value in values {
        if seen.insert(value.clone()) {
            deduped.push(value);
        }
    }
    deduped
}

fn draw_waiting_thin_client(
    canvas: &mut Canvas<Window>,
    active_theme: &crate::theme::Theme,
    connect_addr: &str,
    client_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = canvas.output_size()?;
    canvas.set_draw_color(active_theme.app_chrome.window_clear);
    canvas.clear();

    let outer = Rect::new(12, 12, width.saturating_sub(24), height.saturating_sub(24));
    canvas.set_draw_color(active_theme.app_chrome.surface_fill);
    canvas.fill_rect(outer)?;
    canvas.set_draw_color(active_theme.app_chrome.surface_border);
    canvas.draw_rect(outer)?;

    crate::ui::draw_text(
        canvas,
        "THIN CLIENT",
        outer.x + 12,
        outer.y + 12,
        2,
        active_theme.app_chrome.tab_text_active,
    )?;
    crate::ui::draw_text_fitted(
        canvas,
        &format!("host {connect_addr}"),
        Rect::new(
            outer.x + 12,
            outer.y + 36,
            outer.width().saturating_sub(24),
            8,
        ),
        1,
        active_theme.app_chrome.detail_text,
    )?;
    crate::ui::draw_text_fitted(
        canvas,
        &format!("client {client_name}"),
        Rect::new(
            outer.x + 12,
            outer.y + 48,
            outer.width().saturating_sub(24),
            8,
        ),
        1,
        active_theme.app_chrome.detail_text,
    )?;

    let summary = Rect::new(
        outer.x + 12,
        outer.y + 72,
        outer.width().saturating_sub(24),
        72,
    );
    canvas.set_draw_color(active_theme.app_chrome.surface_fill);
    canvas.fill_rect(summary)?;
    canvas.set_draw_color(active_theme.app_chrome.surface_border);
    canvas.draw_rect(summary)?;

    crate::ui::draw_text_fitted(
        canvas,
        "Waiting for first host snapshot...",
        Rect::new(
            outer.x + 12,
            outer.y + 96,
            outer.width().saturating_sub(24),
            8,
        ),
        1,
        active_theme.app_chrome.detail_text,
    )?;
    crate::ui::draw_text_fitted(
        canvas,
        "Esc quits the thin client locally; other inputs forward to the host.",
        Rect::new(
            outer.x + 12,
            outer.y + 112,
            outer.width().saturating_sub(24),
            8,
        ),
        1,
        active_theme.app_chrome.detail_text,
    )?;

    canvas.present();
    Ok(())
}

fn spawn_stdin_reader(stdin_tx: Sender<String>) {
    thread::spawn(move || {
        let stdin = io::stdin();
        let reader = stdin.lock();
        for line in reader.lines() {
            let Ok(line) = line else {
                break;
            };
            if stdin_tx.send(line).is_err() {
                break;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{
        DISCOVERY_PROTOCOL_VERSION, DiscoveryAdvertisement, SessionCommand,
        choose_preferred_connect_addr,
    };

    #[test]
    fn session_command_parses_common_tokens() {
        assert_eq!(
            SessionCommand::parse_token("space"),
            Some(SessionCommand::TogglePlayback)
        );
        assert_eq!(
            SessionCommand::parse_token("right"),
            Some(SessionCommand::SelectNextTrack)
        );
        assert_eq!(
            SessionCommand::parse_token("mute"),
            Some(SessionCommand::ToggleCurrentTrackMute)
        );
        assert_eq!(SessionCommand::parse_token("nope"), None);
    }

    #[test]
    fn advertised_private_address_beats_link_local_responder_address() {
        let advertisement = DiscoveryAdvertisement {
            session_id: "host:1234".to_string(),
            session_name: "Demo".to_string(),
            host_name: "remote-host".to_string(),
            listen_addr: "0.0.0.0:1234".to_string(),
            connect_addrs: vec![
                "192.168.1.44:1234".to_string(),
                "169.254.12.9:1234".to_string(),
            ],
            port: 1234,
            protocol_version: DISCOVERY_PROTOCOL_VERSION,
            host_mode: "host-session".to_string(),
            current_client_count: 0,
        };

        let chosen = choose_preferred_connect_addr(&advertisement, "169.254.12.9".to_string());
        assert_eq!(chosen.as_deref(), Some("192.168.1.44:1234"));
    }
}
