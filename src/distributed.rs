use crate::actions::{ActionSource, AppAction};
use crate::app::App;
use crate::project::Project;
use crate::theme::{ThemePreset, theme};
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::rect::Rect;
use sdl3::render::Canvas;
use sdl3::video::Window;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
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
    pub transport_ticks: u64,
    pub playhead_ticks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ThinClientMessage {
    Hello { client_name: String },
    Command { command: SessionCommand },
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
struct ReceivedCommand {
    command: SessionCommand,
    client_name: String,
}

pub struct SessionServer {
    command_rx: Receiver<ReceivedCommand>,
    clients: Arc<Mutex<Vec<Sender<HostMessage>>>>,
    revision: u64,
    last_snapshot_json: Option<String>,
    last_snapshot_broadcast_at: Instant,
    _accept_thread: thread::JoinHandle<()>,
}

impl SessionServer {
    pub fn bind(listen_addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(listen_addr)?;
        let (command_tx, command_rx) = mpsc::channel();
        let clients = Arc::new(Mutex::new(Vec::<Sender<HostMessage>>::new()));
        let accept_clients = Arc::clone(&clients);
        let accept_thread = thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else {
                    continue;
                };
                let Ok(write_stream) = stream.try_clone() else {
                    continue;
                };
                let (outbound_tx, outbound_rx) = mpsc::channel::<HostMessage>();
                if let Ok(mut guard) = accept_clients.lock() {
                    guard.push(outbound_tx);
                }
                spawn_client_writer(write_stream, outbound_rx);
                spawn_client_reader(stream, command_tx.clone());
            }
        });

        Ok(Self {
            command_rx,
            clients,
            revision: 0,
            last_snapshot_json: None,
            last_snapshot_broadcast_at: Instant::now() - SNAPSHOT_BROADCAST_INTERVAL,
            _accept_thread: accept_thread,
        })
    }

    pub fn service_app(&mut self, app: &mut App) {
        let mut accepted_commands = Vec::new();
        for received in self.command_rx.try_iter() {
            if app.apply_session_command(received.command, ActionSource::Remote) {
                self.revision = self.revision.saturating_add(1);
                accepted_commands.push((received.client_name, received.command));
            }
        }

        for (_, command) in accepted_commands {
            self.broadcast_message(HostMessage::Ack {
                revision: self.revision,
                command,
            });
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

    fn broadcast_message(&self, message: HostMessage) {
        let Ok(mut guard) = self.clients.lock() else {
            return;
        };
        guard.retain(|client| client.send(message.clone()).is_ok());
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

fn spawn_client_reader(stream: TcpStream, command_tx: Sender<ReceivedCommand>) {
    thread::spawn(move || {
        let mut client_name = format!("client-{}", std::process::id());
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
                    client_name = hello_name;
                }
                ThinClientMessage::Command { command } => {
                    if command_tx
                        .send(ReceivedCommand {
                            command,
                            client_name: client_name.clone(),
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
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
