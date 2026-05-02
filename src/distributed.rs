use crate::actions::{ActionSource, AppAction};
use crate::app::{App, AppControl, ClientUiState};
use crate::link::LinkSnapshot;
use crate::mapping::MappingEntry;
use crate::midi_io::MidiDeviceCatalog;
use crate::project::Project;
use crate::theme::{ThemePreset, theme};
use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Mod};
use sdl3::rect::Rect;
use sdl3::render::Canvas;
use sdl3::video::Window;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const SNAPSHOT_BROADCAST_INTERVAL: Duration = Duration::from_millis(100);
const HEADLESS_STATUS_INTERVAL: Duration = Duration::from_millis(500);
const IO_POLL_INTERVAL: Duration = Duration::from_millis(16);
const SDL_CLIENT_FRAME_INTERVAL: Duration = Duration::from_millis(16);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemotePointerSource {
    Pointer,
    Touch,
}

impl RemotePointerSource {
    fn action_source(self) -> ActionSource {
        match self {
            RemotePointerSource::Pointer => ActionSource::Pointer,
            RemotePointerSource::Touch => ActionSource::Touch,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemoteKeycode {
    Escape,
    Space,
    Return,
    Backspace,
    Delete,
    Tab,
    Home,
    Left,
    Right,
    Up,
    Down,
    Comma,
    Period,
    Minus,
    Equals,
    LeftBracket,
    RightBracket,
    Slash,
    Backslash,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    LShift,
    RShift,
    LCtrl,
    RCtrl,
    LAlt,
    RAlt,
    LGui,
    RGui,
    Mode,
}

impl RemoteKeycode {
    #[allow(dead_code)]
    fn from_sdl(keycode: Keycode) -> Option<Self> {
        Some(match keycode {
            Keycode::Escape => Self::Escape,
            Keycode::Space => Self::Space,
            Keycode::Return => Self::Return,
            Keycode::Backspace => Self::Backspace,
            Keycode::Delete => Self::Delete,
            Keycode::Tab => Self::Tab,
            Keycode::Home => Self::Home,
            Keycode::Left => Self::Left,
            Keycode::Right => Self::Right,
            Keycode::Up => Self::Up,
            Keycode::Down => Self::Down,
            Keycode::Comma => Self::Comma,
            Keycode::Period => Self::Period,
            Keycode::Minus => Self::Minus,
            Keycode::Equals => Self::Equals,
            Keycode::LeftBracket => Self::LeftBracket,
            Keycode::RightBracket => Self::RightBracket,
            Keycode::Slash => Self::Slash,
            Keycode::Backslash => Self::Backslash,
            Keycode::F1 => Self::F1,
            Keycode::F2 => Self::F2,
            Keycode::F3 => Self::F3,
            Keycode::F4 => Self::F4,
            Keycode::F5 => Self::F5,
            Keycode::F6 => Self::F6,
            Keycode::F7 => Self::F7,
            Keycode::F8 => Self::F8,
            Keycode::A => Self::A,
            Keycode::B => Self::B,
            Keycode::C => Self::C,
            Keycode::D => Self::D,
            Keycode::E => Self::E,
            Keycode::F => Self::F,
            Keycode::G => Self::G,
            Keycode::H => Self::H,
            Keycode::I => Self::I,
            Keycode::J => Self::J,
            Keycode::K => Self::K,
            Keycode::L => Self::L,
            Keycode::M => Self::M,
            Keycode::N => Self::N,
            Keycode::O => Self::O,
            Keycode::P => Self::P,
            Keycode::Q => Self::Q,
            Keycode::R => Self::R,
            Keycode::S => Self::S,
            Keycode::T => Self::T,
            Keycode::U => Self::U,
            Keycode::V => Self::V,
            Keycode::W => Self::W,
            Keycode::X => Self::X,
            Keycode::Y => Self::Y,
            Keycode::Z => Self::Z,
            Keycode::_0 => Self::Num0,
            Keycode::_1 => Self::Num1,
            Keycode::_2 => Self::Num2,
            Keycode::_3 => Self::Num3,
            Keycode::_4 => Self::Num4,
            Keycode::_5 => Self::Num5,
            Keycode::_6 => Self::Num6,
            Keycode::_7 => Self::Num7,
            Keycode::_8 => Self::Num8,
            Keycode::_9 => Self::Num9,
            Keycode::Kp1 => Self::Kp1,
            Keycode::Kp2 => Self::Kp2,
            Keycode::Kp3 => Self::Kp3,
            Keycode::Kp4 => Self::Kp4,
            Keycode::Kp5 => Self::Kp5,
            Keycode::Kp6 => Self::Kp6,
            Keycode::Kp7 => Self::Kp7,
            Keycode::Kp8 => Self::Kp8,
            Keycode::LShift => Self::LShift,
            Keycode::RShift => Self::RShift,
            Keycode::LCtrl => Self::LCtrl,
            Keycode::RCtrl => Self::RCtrl,
            Keycode::LAlt => Self::LAlt,
            Keycode::RAlt => Self::RAlt,
            Keycode::LGui => Self::LGui,
            Keycode::RGui => Self::RGui,
            Keycode::Mode => Self::Mode,
            _ => return None,
        })
    }

    fn to_sdl(self) -> Keycode {
        match self {
            Self::Escape => Keycode::Escape,
            Self::Space => Keycode::Space,
            Self::Return => Keycode::Return,
            Self::Backspace => Keycode::Backspace,
            Self::Delete => Keycode::Delete,
            Self::Tab => Keycode::Tab,
            Self::Home => Keycode::Home,
            Self::Left => Keycode::Left,
            Self::Right => Keycode::Right,
            Self::Up => Keycode::Up,
            Self::Down => Keycode::Down,
            Self::Comma => Keycode::Comma,
            Self::Period => Keycode::Period,
            Self::Minus => Keycode::Minus,
            Self::Equals => Keycode::Equals,
            Self::LeftBracket => Keycode::LeftBracket,
            Self::RightBracket => Keycode::RightBracket,
            Self::Slash => Keycode::Slash,
            Self::Backslash => Keycode::Backslash,
            Self::F1 => Keycode::F1,
            Self::F2 => Keycode::F2,
            Self::F3 => Keycode::F3,
            Self::F4 => Keycode::F4,
            Self::F5 => Keycode::F5,
            Self::F6 => Keycode::F6,
            Self::F7 => Keycode::F7,
            Self::F8 => Keycode::F8,
            Self::A => Keycode::A,
            Self::B => Keycode::B,
            Self::C => Keycode::C,
            Self::D => Keycode::D,
            Self::E => Keycode::E,
            Self::F => Keycode::F,
            Self::G => Keycode::G,
            Self::H => Keycode::H,
            Self::I => Keycode::I,
            Self::J => Keycode::J,
            Self::K => Keycode::K,
            Self::L => Keycode::L,
            Self::M => Keycode::M,
            Self::N => Keycode::N,
            Self::O => Keycode::O,
            Self::P => Keycode::P,
            Self::Q => Keycode::Q,
            Self::R => Keycode::R,
            Self::S => Keycode::S,
            Self::T => Keycode::T,
            Self::U => Keycode::U,
            Self::V => Keycode::V,
            Self::W => Keycode::W,
            Self::X => Keycode::X,
            Self::Y => Keycode::Y,
            Self::Z => Keycode::Z,
            Self::Num0 => Keycode::_0,
            Self::Num1 => Keycode::_1,
            Self::Num2 => Keycode::_2,
            Self::Num3 => Keycode::_3,
            Self::Num4 => Keycode::_4,
            Self::Num5 => Keycode::_5,
            Self::Num6 => Keycode::_6,
            Self::Num7 => Keycode::_7,
            Self::Num8 => Keycode::_8,
            Self::Num9 => Keycode::_9,
            Self::Kp1 => Keycode::Kp1,
            Self::Kp2 => Keycode::Kp2,
            Self::Kp3 => Keycode::Kp3,
            Self::Kp4 => Keycode::Kp4,
            Self::Kp5 => Keycode::Kp5,
            Self::Kp6 => Keycode::Kp6,
            Self::Kp7 => Keycode::Kp7,
            Self::Kp8 => Keycode::Kp8,
            Self::LShift => Keycode::LShift,
            Self::RShift => Keycode::RShift,
            Self::LCtrl => Keycode::LCtrl,
            Self::RCtrl => Keycode::RCtrl,
            Self::LAlt => Keycode::LAlt,
            Self::RAlt => Keycode::RAlt,
            Self::LGui => Keycode::LGui,
            Self::RGui => Keycode::RGui,
            Self::Mode => Keycode::Mode,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RemoteInputEvent {
    KeyDown {
        keycode: RemoteKeycode,
        keymod_bits: u16,
        repeat: bool,
    },
    PointerHover {
        x: i32,
        y: i32,
    },
    PointerDown {
        x: i32,
        y: i32,
        source: RemotePointerSource,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ThinClientMessage {
    Hello { client_name: String },
    Command { command: SessionCommand },
    Input { input: RemoteInputEvent },
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
    Connected { client_id: usize },
    Disconnected { client_id: usize },
    Hello { client_id: usize, client_name: String },
    Command { command: SessionCommand },
    Input { client_id: usize, input: RemoteInputEvent },
}

#[derive(Debug, Clone)]
struct ClientContext {
    client_name: String,
    ui_state: ClientUiState,
}

pub struct SessionServer {
    command_rx: Receiver<ReceivedClientMessage>,
    clients: Arc<Mutex<HashMap<usize, Sender<HostMessage>>>>,
    contexts: HashMap<usize, ClientContext>,
    revision: u64,
    last_snapshot_json: Option<String>,
    last_snapshot_broadcast_at: Instant,
    _accept_thread: thread::JoinHandle<()>,
}

impl SessionServer {
    pub fn bind(listen_addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(listen_addr)?;
        let (command_tx, command_rx) = mpsc::channel();
        let clients = Arc::new(Mutex::new(HashMap::<usize, Sender<HostMessage>>::new()));
        let accept_clients = Arc::clone(&clients);
        let next_client_id = Arc::new(AtomicUsize::new(1));
        let accept_next_client_id = Arc::clone(&next_client_id);
        let accept_thread = thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else {
                    continue;
                };
                let client_id = accept_next_client_id.fetch_add(1, Ordering::Relaxed);
                let Ok(write_stream) = stream.try_clone() else {
                    continue;
                };
                let (outbound_tx, outbound_rx) = mpsc::channel::<HostMessage>();
                if let Ok(mut guard) = accept_clients.lock() {
                    guard.insert(client_id, outbound_tx);
                }
                let _ = command_tx.send(ReceivedClientMessage::Connected { client_id });
                spawn_client_writer(write_stream, outbound_rx);
                spawn_client_reader(stream, client_id, command_tx.clone());
            }
        });

        Ok(Self {
            command_rx,
            clients,
            contexts: HashMap::new(),
            revision: 0,
            last_snapshot_json: None,
            last_snapshot_broadcast_at: Instant::now() - SNAPSHOT_BROADCAST_INTERVAL,
            _accept_thread: accept_thread,
        })
    }

    pub fn service_app(&mut self, app: &mut App) {
        let received_messages: Vec<_> = self.command_rx.try_iter().collect();
        for received in received_messages {
            match received {
                ReceivedClientMessage::Connected { client_id } => {
                    self.contexts.entry(client_id).or_insert_with(|| ClientContext {
                        client_name: format!("client-{client_id}"),
                        ui_state: ClientUiState::default(),
                    });
                }
                ReceivedClientMessage::Disconnected { client_id } => {
                    self.contexts.remove(&client_id);
                    if let Ok(mut guard) = self.clients.lock() {
                        guard.remove(&client_id);
                    }
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
                        self.revision = self.revision.saturating_add(1);
                        self.broadcast_message(HostMessage::Ack {
                            revision: self.revision,
                            command,
                        });
                    }
                }
                ReceivedClientMessage::Input { client_id, input } => {
                    if self.apply_remote_input(app, client_id, input) {
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
        self.clients.lock().map(|guard| guard.len()).unwrap_or(0)
    }

    fn apply_remote_input(
        &mut self,
        app: &mut App,
        client_id: usize,
        input: RemoteInputEvent,
    ) -> bool {
        let context = self.contexts.entry(client_id).or_insert_with(|| ClientContext {
            client_name: format!("client-{client_id}"),
            ui_state: ClientUiState::default(),
        });
        let host_ui_state = app.capture_client_ui_state();
        let before_snapshot = serde_json::to_string(&app.session_snapshot(0, 0)).ok();
        app.apply_client_ui_state(&context.ui_state);

        let control = match input {
            RemoteInputEvent::KeyDown {
                keycode,
                keymod_bits,
                repeat,
            } => app.handle_remote_key_down(
                keycode.to_sdl(),
                Mod::from_bits_truncate(keymod_bits),
                repeat,
            ),
            RemoteInputEvent::PointerHover { x, y } => Some(app.handle_remote_pointer_hover(x, y)),
            RemoteInputEvent::PointerDown { x, y, source } => {
                app.handle_remote_pointer_down(x, y, source.action_source())
            }
        };
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
                ThinClientMessage::Input { input } => {
                    if command_tx
                        .send(ReceivedClientMessage::Input { client_id, input })
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
    let mut server = SessionServer::bind(listen_addr)?;
    let (stdin_tx, stdin_rx) = mpsc::channel::<String>();
    spawn_stdin_reader(stdin_tx);

    println!("trekr headless session host listening on {listen_addr}");
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

pub fn run_thin_client_sdl(
    connect_addr: &str,
    client_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut writer, snapshot_rx) = connect_thin_client_channel(connect_addr, client_name)?;
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
    canvas.set_scale(1.0, 1.0)?;
    let mut event_pump = sdl_context.event_pump()?;
    let active_theme = theme(ThemePreset::DefaultDark);
    let mut latest_snapshot: Option<SessionSnapshot> = None;
    let mut status_line = format!("Connected to {connect_addr} as {client_name}");

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    repeat: false,
                    ..
                } => {
                    if keycode == Keycode::Escape {
                        break 'running;
                    }
                    if let Some(command) = session_command_for_key(keycode) {
                        send_thin_client_command(&mut writer, command)?;
                        status_line = format!("Sent {}", command.label());
                    }
                }
                Event::MouseButtonDown { x, y, .. } => {
                    if let Some((snapshot, command)) = button_command_at(
                        latest_snapshot.as_ref(),
                        canvas.output_size().unwrap_or((960, 540)),
                        x,
                        y,
                    ) {
                        let _ = snapshot;
                        send_thin_client_command(&mut writer, command)?;
                        status_line = format!("Sent {}", command.label());
                    }
                }
                _ => {}
            }
        }

        for message in snapshot_rx.try_iter() {
            match message {
                HostMessage::Snapshot { snapshot } => latest_snapshot = Some(snapshot),
                HostMessage::Ack { revision, command } => {
                    status_line = format!("Ack rev {revision} for {}", command.label())
                }
                HostMessage::Reject { message } => status_line = format!("Rejected: {message}"),
            }
        }

        draw_sdl_thin_client(
            &mut canvas,
            active_theme,
            connect_addr,
            client_name,
            latest_snapshot.as_ref(),
            &status_line,
        )?;
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

fn send_thin_client_command(
    writer: &mut TcpStream,
    command: SessionCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(
        writer,
        "{}",
        serde_json::to_string(&ThinClientMessage::Command { command })?
    )?;
    writer.flush()?;
    Ok(())
}

fn session_command_for_key(keycode: Keycode) -> Option<SessionCommand> {
    match keycode {
        Keycode::Space => Some(SessionCommand::TogglePlayback),
        Keycode::R => Some(SessionCommand::ToggleRecording),
        Keycode::G => Some(SessionCommand::ToggleGlobalLoop),
        Keycode::Home => Some(SessionCommand::ResetGlobalLoop),
        Keycode::Right => Some(SessionCommand::SelectNextTrack),
        Keycode::Left => Some(SessionCommand::SelectPreviousTrack),
        Keycode::A => Some(SessionCommand::ToggleCurrentTrackArm),
        Keycode::M => Some(SessionCommand::ToggleCurrentTrackMute),
        Keycode::S => Some(SessionCommand::ToggleCurrentTrackSolo),
        Keycode::I => Some(SessionCommand::ToggleCurrentTrackPassthrough),
        _ => None,
    }
}

fn button_command_at(
    snapshot: Option<&SessionSnapshot>,
    viewport: (u32, u32),
    x: f32,
    y: f32,
) -> Option<(usize, SessionCommand)> {
    let _ = snapshot?;
    let buttons = command_buttons(viewport);
    let xi = x as i32;
    let yi = y as i32;
    for (index, (rect, command)) in buttons.into_iter().enumerate() {
        if point_in_rect(rect, xi, yi) {
            return Some((index, command));
        }
    }
    None
}

fn draw_sdl_thin_client(
    canvas: &mut Canvas<Window>,
    active_theme: &crate::theme::Theme,
    connect_addr: &str,
    client_name: &str,
    snapshot: Option<&SessionSnapshot>,
    status_line: &str,
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
        120,
    );
    canvas.set_draw_color(active_theme.app_chrome.surface_fill);
    canvas.fill_rect(summary)?;
    canvas.set_draw_color(active_theme.app_chrome.surface_border);
    canvas.draw_rect(summary)?;

    if let Some(snapshot) = snapshot {
        let active_track = snapshot
            .project
            .tracks
            .get(snapshot.project.active_track_index)
            .expect("project always has active track");
        crate::ui::draw_text_fitted(
            canvas,
            &format!(
                "rev {}  clients {}",
                snapshot.revision, snapshot.connected_clients
            ),
            Rect::new(
                summary.x + 8,
                summary.y + 8,
                summary.width().saturating_sub(16),
                8,
            ),
            1,
            active_theme.app_chrome.action_text,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            &format!(
                "transport playing={} recording={} loop={} tempo={}bpm",
                on_off(snapshot.project.transport.playing),
                on_off(snapshot.project.transport.recording),
                on_off(snapshot.project.transport.loop_enabled),
                snapshot.project.transport.tempo_bpm
            ),
            Rect::new(
                summary.x + 8,
                summary.y + 22,
                summary.width().saturating_sub(16),
                8,
            ),
            1,
            active_theme.app_chrome.action_text,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            &format!(
                "ticks transport={} playhead={}",
                snapshot.transport_ticks, snapshot.playhead_ticks
            ),
            Rect::new(
                summary.x + 8,
                summary.y + 36,
                summary.width().saturating_sub(16),
                8,
            ),
            1,
            active_theme.app_chrome.action_text,
        )?;
        crate::ui::draw_text_fitted(
            canvas,
            &format!(
                "track {} {} arm={} mute={} solo={} thru={}",
                snapshot.project.active_track_index + 1,
                active_track.name,
                on_off(active_track.state.armed),
                on_off(active_track.state.muted),
                on_off(active_track.state.soloed),
                on_off(active_track.state.passthrough)
            ),
            Rect::new(
                summary.x + 8,
                summary.y + 50,
                summary.width().saturating_sub(16),
                8,
            ),
            1,
            active_theme.app_chrome.action_text,
        )?;
    } else {
        crate::ui::draw_text_fitted(
            canvas,
            "Waiting for first snapshot...",
            Rect::new(
                summary.x + 8,
                summary.y + 8,
                summary.width().saturating_sub(16),
                8,
            ),
            1,
            active_theme.app_chrome.detail_text,
        )?;
    }

    for (rect, command) in command_buttons((width, height)) {
        let active = snapshot
            .and_then(|snapshot| is_command_active(snapshot, command))
            .unwrap_or(false);
        let fill = if active {
            active_theme.transport.play_active
        } else {
            active_theme.app_chrome.surface_fill
        };
        let text = if active {
            active_theme.text_on_fill(fill)
        } else {
            active_theme.app_chrome.action_text
        };
        canvas.set_draw_color(fill);
        canvas.fill_rect(rect)?;
        canvas.set_draw_color(active_theme.app_chrome.surface_border);
        canvas.draw_rect(rect)?;
        crate::ui::draw_text_fitted(
            canvas,
            command.label(),
            Rect::new(rect.x + 4, rect.y + 8, rect.width().saturating_sub(8), 8),
            1,
            text,
        )?;
    }

    crate::ui::draw_text_fitted(
        canvas,
        "Keys: Space R G Home Left Right A M S I  Esc quits",
        Rect::new(
            outer.x + 12,
            outer.y + outer.height() as i32 - 36,
            outer.width().saturating_sub(24),
            8,
        ),
        1,
        active_theme.app_chrome.detail_text,
    )?;
    crate::ui::draw_text_fitted(
        canvas,
        status_line,
        Rect::new(
            outer.x + 12,
            outer.y + outer.height() as i32 - 20,
            outer.width().saturating_sub(24),
            8,
        ),
        1,
        active_theme.app_chrome.action_text,
    )?;

    canvas.present();
    Ok(())
}

fn command_buttons(viewport: (u32, u32)) -> Vec<(Rect, SessionCommand)> {
    let width = viewport.0.saturating_sub(48);
    let start_x = 24;
    let start_y = 220;
    let columns = 2_i32;
    let gap = 12_i32;
    let cell_width = ((width as i32) - gap) / columns;
    let cell_height = 34_u32;
    SessionCommand::ALL
        .iter()
        .enumerate()
        .map(|(index, command)| {
            let row = index as i32 / columns;
            let col = index as i32 % columns;
            let rect = Rect::new(
                start_x + col * (cell_width + gap),
                start_y + row * (cell_height as i32 + gap),
                cell_width.max(120) as u32,
                cell_height,
            );
            (rect, *command)
        })
        .collect()
}

fn point_in_rect(rect: Rect, x: i32, y: i32) -> bool {
    x >= rect.x
        && y >= rect.y
        && x < rect.x + rect.width() as i32
        && y < rect.y + rect.height() as i32
}

fn is_command_active(snapshot: &SessionSnapshot, command: SessionCommand) -> Option<bool> {
    let active_track = snapshot
        .project
        .tracks
        .get(snapshot.project.active_track_index)?;
    Some(match command {
        SessionCommand::TogglePlayback => snapshot.project.transport.playing,
        SessionCommand::ToggleRecording => snapshot.project.transport.recording,
        SessionCommand::ToggleGlobalLoop => snapshot.project.transport.loop_enabled,
        SessionCommand::ResetGlobalLoop => false,
        SessionCommand::SelectNextTrack | SessionCommand::SelectPreviousTrack => false,
        SessionCommand::ToggleCurrentTrackArm => active_track.state.armed,
        SessionCommand::ToggleCurrentTrackMute => active_track.state.muted,
        SessionCommand::ToggleCurrentTrackSolo => active_track.state.soloed,
        SessionCommand::ToggleCurrentTrackPassthrough => active_track.state.passthrough,
    })
}

fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
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
    use super::SessionCommand;

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
}
