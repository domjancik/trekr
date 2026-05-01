use crate::app::{App, RunOptions, UiCaptureOptions, UiScalingMode, VideoMode};
use crate::distributed;
use crate::state;
use crate::theme::ThemePreset;
use crate::ui_density::UiDensityPreset;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateMode {
    Persisted,
    Demo,
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaunchOptions {
    pub run_mode: LaunchMode,
    pub state_mode: StateMode,
    pub state_file: PathBuf,
    pub ui_scale: Option<f32>,
    pub ui_scaling_mode: UiScalingMode,
    pub theme_preset: Option<ThemePreset>,
    pub ui_density_preset: Option<UiDensityPreset>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LaunchMode {
    Interactive(RunOptions),
    Capture(UiCaptureOptions),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppCommand {
    Launch(LaunchOptions),
    HostSession(HostSessionOptions),
    ThinClient(ThinClientOptions),
    PrintHelp,
    PrintCommands,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HostSessionOptions {
    pub launch: LaunchOptions,
    pub listen_addr: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThinClientOptions {
    pub connect_addr: String,
    pub client_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuggestedCommand {
    pub label: &'static str,
    pub command: &'static str,
    pub description: &'static str,
    pub args: &'static [&'static str],
    pub launchable: bool,
}

pub const DEFAULT_STATE_FILE: &str = "artifacts/state/last-run.json";
pub const DEFAULT_CAPTURE_DIR: &str = "artifacts/screenshots";

const SUGGESTED_COMMANDS: [SuggestedCommand; 8] = [
    SuggestedCommand {
        label: "Desktop persisted session",
        command: "cargo run -- run",
        description: "Use the last persisted state when available and save back on clean exit.",
        args: &["run"],
        launchable: true,
    },
    SuggestedCommand {
        label: "Desktop demo session",
        command: "cargo run -- run --state-mode demo",
        description: "Start from the built-in deterministic demo project.",
        args: &["run", "--state-mode", "demo"],
        launchable: true,
    },
    SuggestedCommand {
        label: "Desktop empty session",
        command: "cargo run -- run --state-mode empty",
        description: "Start from a deterministic empty project.",
        args: &["run", "--state-mode", "empty"],
        launchable: true,
    },
    SuggestedCommand {
        label: "KMSDRM console session",
        command: "cargo run -- run --state-mode demo --video-mode kmsdrm-console",
        description: "Use the direct Linux console video backend for Raspberry Pi style targets.",
        args: &[
            "run",
            "--state-mode",
            "demo",
            "--video-mode",
            "kmsdrm-console",
        ],
        launchable: true,
    },
    SuggestedCommand {
        label: "Renderer screenshot capture",
        command: "cargo run -- capture-ui --state-mode demo --capture-dir artifacts/screenshots",
        description: "Render deterministic UI screenshots from the app itself.",
        args: &[
            "capture-ui",
            "--state-mode",
            "demo",
            "--capture-dir",
            "artifacts/screenshots",
        ],
        launchable: true,
    },
    SuggestedCommand {
        label: "Headless session host",
        command: "cargo run -- host-session --state-mode demo --listen 0.0.0.0:8787",
        description: "Run the shared session headlessly and accept thin-client connections over TCP.",
        args: &[
            "host-session",
            "--state-mode",
            "demo",
            "--listen",
            "0.0.0.0:8787",
        ],
        launchable: true,
    },
    SuggestedCommand {
        label: "Terminal thin client",
        command: "cargo run -- thin-client --connect 127.0.0.1:8787",
        description: "Connect a lightweight terminal control surface to a session host.",
        args: &["thin-client", "--connect", "127.0.0.1:8787"],
        launchable: true,
    },
    SuggestedCommand {
        label: "Terminal launch picker",
        command: "cargo run --bin trekr-tui",
        description: "Open the text UI selector for common launch profiles.",
        args: &[],
        launchable: false,
    },
];

pub fn parse_app_command() -> Result<AppCommand, String> {
    parse_app_command_from(std::env::args().skip(1))
}

pub fn parse_app_command_from<I>(args: I) -> Result<AppCommand, String>
where
    I: IntoIterator<Item = String>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    let Some(first) = args.first() else {
        return Ok(AppCommand::Launch(LaunchOptions::default()));
    };

    match first.as_str() {
        "help" | "--help" | "-h" => Ok(AppCommand::PrintHelp),
        "commands" => Ok(AppCommand::PrintCommands),
        "run"
            if args
                .iter()
                .skip(1)
                .any(|arg| arg == "--help" || arg == "-h") =>
        {
            Ok(AppCommand::PrintHelp)
        }
        "run" => parse_launch_options_from(args.into_iter().skip(1), false).map(AppCommand::Launch),
        "capture-ui"
            if args
                .iter()
                .skip(1)
                .any(|arg| arg == "--help" || arg == "-h") =>
        {
            Ok(AppCommand::PrintHelp)
        }
        "capture-ui" => {
            parse_launch_options_from(args.into_iter().skip(1), true).map(AppCommand::Launch)
        }
        "host-session"
            if args
                .iter()
                .skip(1)
                .any(|arg| arg == "--help" || arg == "-h") =>
        {
            Ok(AppCommand::PrintHelp)
        }
        "host-session" => {
            parse_host_session_options_from(args.into_iter().skip(1)).map(AppCommand::HostSession)
        }
        "thin-client"
            if args
                .iter()
                .skip(1)
                .any(|arg| arg == "--help" || arg == "-h") =>
        {
            Ok(AppCommand::PrintHelp)
        }
        "thin-client" => {
            parse_thin_client_options_from(args.into_iter().skip(1)).map(AppCommand::ThinClient)
        }
        _ if first.starts_with('-') => {
            parse_launch_options_from(args.into_iter(), false).map(AppCommand::Launch)
        }
        other => Err(format!("unknown command: {other}")),
    }
}

impl Default for LaunchOptions {
    fn default() -> Self {
        Self {
            run_mode: LaunchMode::Interactive(RunOptions::default()),
            state_mode: StateMode::Persisted,
            state_file: PathBuf::from(DEFAULT_STATE_FILE),
            ui_scale: None,
            ui_scaling_mode: UiScalingMode::Auto,
            theme_preset: None,
            ui_density_preset: None,
        }
    }
}

pub fn execute_app_command(command: AppCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        AppCommand::Launch(options) => launch(options),
        AppCommand::HostSession(options) => host_session(options),
        AppCommand::ThinClient(options) => {
            distributed::run_thin_client(&options.connect_addr, &options.client_name)
        }
        AppCommand::PrintHelp => {
            print_help(&mut io::stdout())?;
            Ok(())
        }
        AppCommand::PrintCommands => {
            print_suggested_commands(&mut io::stdout())?;
            Ok(())
        }
    }
}

pub fn launch(options: LaunchOptions) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = build_app(&options);
    println!("{}", app.bootstrap_summary());
    match options.run_mode {
        LaunchMode::Interactive(run_options) => {
            let result = app.run_with_options(run_options);
            if result.is_ok() && options.state_mode == StateMode::Persisted {
                state::save(&options.state_file, &app.persisted_state())?;
                let undo_path = state::undo_history_path(&options.state_file);
                state::save_undo_history(&undo_path, app.undo_history())?;
            }
            result
        }
        LaunchMode::Capture(capture) => {
            if options.state_mode == StateMode::Demo {
                app.seed_capture_demo_timeline_overlaps();
            }
            app.capture_ui_pages(capture)
        }
    }
}

pub fn host_session(options: HostSessionOptions) -> Result<(), Box<dyn std::error::Error>> {
    let app = build_app(&options.launch);
    println!("{}", app.bootstrap_summary());
    distributed::run_headless_session_host(app, &options.listen_addr)
}

fn build_app(options: &LaunchOptions) -> App {
    let mut app = match options.state_mode {
        StateMode::Persisted => {
            if options.state_file.exists() {
                match state::load(&options.state_file) {
                    Ok(state) => {
                        let mut app = App::from_persisted_state(state);
                        let undo_path = state::undo_history_path(&options.state_file);
                        app.set_undo_history(state::load_undo_history(&undo_path));
                        app
                    }
                    Err(_) => App::new_demo(),
                }
            } else {
                App::new_demo()
            }
        }
        StateMode::Demo => App::new_demo(),
        StateMode::Empty => App::new_empty(),
    };
    app.set_ui_scale_override(options.ui_scale);
    app.set_ui_scaling_mode(options.ui_scaling_mode);
    if let Some(theme_preset) = options.theme_preset {
        app.set_theme_preset(theme_preset);
    }
    if let Some(ui_density_preset) = options.ui_density_preset {
        app.set_ui_density_preset(ui_density_preset);
    }
    app
}

pub fn print_help<W: Write>(writer: &mut W) -> io::Result<()> {
    writeln!(writer, "trekr CLI")?;
    writeln!(writer)?;
    writeln!(writer, "usage: cargo run -- [command] [options]")?;
    writeln!(writer)?;
    writeln!(writer, "commands:")?;
    writeln!(
        writer,
        "  run         launch the SDL app (default when no command is given)"
    )?;
    writeln!(
        writer,
        "  capture-ui  render UI screenshots without opening the interactive app"
    )?;
    writeln!(
        writer,
        "  host-session run a headless shared session host for thin clients"
    )?;
    writeln!(
        writer,
        "  thin-client connect a lightweight terminal control client to a session host"
    )?;
    writeln!(
        writer,
        "  commands    print suggested documented launch commands"
    )?;
    writeln!(writer, "  help        show this help")?;
    writeln!(writer)?;
    writeln!(writer, "options for `run` and `capture-ui`:")?;
    writeln!(writer, "  --state-mode <persisted|demo|empty>")?;
    writeln!(
        writer,
        "  --state-file <path>            default: {DEFAULT_STATE_FILE}"
    )?;
    writeln!(writer, "  --ui-scale <number>=1.0+")?;
    writeln!(
        writer,
        "  --ui-scaling <auto|nearest|linear>   default: auto"
    )?;
    writeln!(
        writer,
        "  --theme <default-dark|high-contrast-dark|high-contrast-light>"
    )?;
    writeln!(
        writer,
        "  --ui-density <default|compact|touch|tiny>   env fallback: TREKR_UI_DENSITY"
    )?;
    writeln!(
        writer,
        "  --video-mode <windowed|fullscreen|kmsdrm-console>   run only"
    )?;
    writeln!(
        writer,
        "  --listen <addr>               run only, expose the SDL app as a thin-client session host"
    )?;
    writeln!(
        writer,
        "  --capture-dir <path>          capture-ui only, default: {DEFAULT_CAPTURE_DIR}"
    )?;
    writeln!(writer)?;
    writeln!(writer, "options for `host-session`:")?;
    writeln!(
        writer,
        "  --listen <addr>               required, for example 0.0.0.0:8787"
    )?;
    writeln!(
        writer,
        "  plus all launch-state options from `run` except --video-mode"
    )?;
    writeln!(writer)?;
    writeln!(writer, "options for `thin-client`:")?;
    writeln!(
        writer,
        "  --connect <addr>              required, for example 127.0.0.1:8787"
    )?;
    writeln!(
        writer,
        "  --name <client-name>          optional, default: thin-client"
    )?;
    writeln!(writer)?;
    writeln!(writer, "compatibility:")?;
    writeln!(
        writer,
        "  legacy flag-only invocation still works, for example `cargo run -- --state-mode demo`"
    )?;
    writeln!(writer)?;
    writeln!(writer, "suggested commands:")?;
    for command in suggested_commands() {
        writeln!(writer, "  {:<28} {}", command.command, command.description)?;
    }
    writeln!(writer)?;
    writeln!(writer, "tui launcher:")?;
    writeln!(
        writer,
        "  cargo run --bin trekr-tui      select a shared launch profile from a terminal menu"
    )?;
    Ok(())
}

pub fn print_suggested_commands<W: Write>(writer: &mut W) -> io::Result<()> {
    writeln!(writer, "Suggested trekr commands")?;
    writeln!(writer)?;
    for command in suggested_commands() {
        writeln!(writer, "{}:", command.label)?;
        writeln!(writer, "  {}", command.command)?;
        writeln!(writer, "  {}", command.description)?;
        writeln!(writer)?;
    }
    Ok(())
}

pub fn suggested_commands() -> &'static [SuggestedCommand] {
    &SUGGESTED_COMMANDS
}

pub fn run_terminal_launcher() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    writeln!(stdout, "trekr terminal launcher")?;
    writeln!(stdout)?;

    loop {
        match prompt_menu(
            &mut stdout,
            &mut stdin,
            "Select an action",
            &[
                "Run interactive app",
                "Capture UI screenshots",
                "Print suggested commands",
                "Print CLI help",
                "Quit",
            ],
            0,
        )? {
            0 => {
                let options = prompt_launch_options(&mut stdout, &mut stdin, false)?;
                print_equivalent_command(&mut stdout, &options)?;
                launch(options)?;
                break;
            }
            1 => {
                let options = prompt_launch_options(&mut stdout, &mut stdin, true)?;
                print_equivalent_command(&mut stdout, &options)?;
                launch(options)?;
                break;
            }
            2 => {
                writeln!(stdout)?;
                print_suggested_commands(&mut stdout)?;
            }
            3 => {
                writeln!(stdout)?;
                print_help(&mut stdout)?;
            }
            4 => break,
            _ => unreachable!(),
        }
        writeln!(stdout)?;
    }

    Ok(())
}

pub fn launch_command_args(options: &LaunchOptions) -> Vec<String> {
    let mut args = Vec::new();

    match &options.run_mode {
        LaunchMode::Interactive(run_options) => {
            args.push("run".to_owned());
            if run_options.video_mode != VideoMode::Windowed {
                args.push("--video-mode".to_owned());
                args.push(video_mode_label(run_options.video_mode).to_owned());
            }
            if let Some(listen_addr) = &run_options.session_listen {
                args.push("--listen".to_owned());
                args.push(listen_addr.clone());
            }
        }
        LaunchMode::Capture(capture) => {
            args.push("capture-ui".to_owned());
            if capture.output_dir != PathBuf::from(DEFAULT_CAPTURE_DIR) {
                args.push("--capture-dir".to_owned());
                args.push(capture.output_dir.display().to_string());
            }
        }
    }

    if options.state_mode != StateMode::Persisted {
        args.push("--state-mode".to_owned());
        args.push(state_mode_label(options.state_mode).to_owned());
    }
    if options.state_file != PathBuf::from(DEFAULT_STATE_FILE) {
        args.push("--state-file".to_owned());
        args.push(options.state_file.display().to_string());
    }
    if let Some(ui_scale) = options.ui_scale {
        args.push("--ui-scale".to_owned());
        args.push(ui_scale.to_string());
    }
    if options.ui_scaling_mode != UiScalingMode::Auto {
        args.push("--ui-scaling".to_owned());
        args.push(ui_scaling_mode_label(options.ui_scaling_mode).to_owned());
    }
    if let Some(theme_preset) = options.theme_preset {
        args.push("--theme".to_owned());
        args.push(theme_preset.label().to_owned());
    }
    if let Some(ui_density_preset) = options.ui_density_preset {
        args.push("--ui-density".to_owned());
        args.push(ui_density_preset.label().to_owned());
    }

    args
}

fn parse_launch_options_from<I>(args: I, capture_mode: bool) -> Result<LaunchOptions, String>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let mut capture_dir = capture_mode.then(|| PathBuf::from(DEFAULT_CAPTURE_DIR));
    let mut options = LaunchOptions::default();
    let mut run_options = RunOptions::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--capture-ui" => {
                if capture_dir.is_none() {
                    capture_dir = Some(PathBuf::from(DEFAULT_CAPTURE_DIR));
                }
            }
            "--capture-dir" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--capture-dir requires a path".to_owned())?;
                capture_dir = Some(PathBuf::from(value));
            }
            "--state-mode" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--state-mode requires persisted|demo|empty".to_owned())?;
                options.state_mode = parse_state_mode(&value)?;
            }
            "--state-file" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--state-file requires a path".to_owned())?;
                options.state_file = PathBuf::from(value);
            }
            "--ui-scale" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--ui-scale requires a numeric value".to_owned())?;
                let parsed = value
                    .parse::<f32>()
                    .map_err(|_| format!("invalid --ui-scale value: {value}"))?;
                if parsed < 1.0 {
                    return Err("--ui-scale must be at least 1.0".to_owned());
                }
                options.ui_scale = Some(parsed);
            }
            "--ui-scaling" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--ui-scaling requires auto|nearest|linear".to_owned())?;
                options.ui_scaling_mode = parse_ui_scaling_mode(&value)?;
            }
            "--theme" => {
                let value = args.next().ok_or_else(|| {
                    "--theme requires default-dark|high-contrast-dark|high-contrast-light"
                        .to_owned()
                })?;
                options.theme_preset = Some(parse_theme_preset(&value)?);
            }
            "--ui-density" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--ui-density requires default|compact|touch|tiny".to_owned())?;
                options.ui_density_preset = Some(parse_ui_density_preset(&value)?);
            }
            "--video-mode" => {
                let value = args.next().ok_or_else(|| {
                    "--video-mode requires windowed|fullscreen|kmsdrm-console".to_owned()
                })?;
                if capture_mode {
                    return Err("--video-mode is only valid with the run command".to_owned());
                }
                run_options.video_mode = parse_video_mode(&value)?;
            }
            "--listen" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--listen requires host:port".to_owned())?;
                if capture_mode {
                    return Err("--listen is only valid with the run command".to_owned());
                }
                run_options.session_listen = Some(value);
            }
            "--help" | "-h" => return Err("use `help` to print the full CLI reference".to_owned()),
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    options.run_mode = match capture_dir {
        Some(output_dir) => LaunchMode::Capture(UiCaptureOptions { output_dir }),
        None => LaunchMode::Interactive(run_options),
    };

    Ok(options)
}

fn parse_host_session_options_from<I>(args: I) -> Result<HostSessionOptions, String>
where
    I: IntoIterator<Item = String>,
{
    let mut listen_addr = None;
    let mut passthrough_args = Vec::new();
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        if arg == "--listen" {
            listen_addr = Some(
                iter.next()
                    .ok_or_else(|| "--listen requires host:port".to_owned())?,
            );
        } else {
            passthrough_args.push(arg);
        }
    }

    let mut launch = parse_launch_options_from(passthrough_args, false)?;
    if let LaunchMode::Interactive(run_options) = &mut launch.run_mode {
        run_options.video_mode = VideoMode::Windowed;
        run_options.session_listen = None;
    }

    Ok(HostSessionOptions {
        launch,
        listen_addr: listen_addr.ok_or_else(|| "--listen is required".to_owned())?,
    })
}

fn parse_thin_client_options_from<I>(args: I) -> Result<ThinClientOptions, String>
where
    I: IntoIterator<Item = String>,
{
    let mut connect_addr = None;
    let mut client_name = "thin-client".to_owned();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--connect" => {
                connect_addr = Some(
                    args.next()
                        .ok_or_else(|| "--connect requires host:port".to_owned())?,
                );
            }
            "--name" => {
                client_name = args
                    .next()
                    .ok_or_else(|| "--name requires a client name".to_owned())?;
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    Ok(ThinClientOptions {
        connect_addr: connect_addr.ok_or_else(|| "--connect is required".to_owned())?,
        client_name,
    })
}

fn parse_video_mode(value: &str) -> Result<VideoMode, String> {
    match value {
        "windowed" => Ok(VideoMode::Windowed),
        "fullscreen" => Ok(VideoMode::Fullscreen),
        "kmsdrm-console" | "kmsdrm" => Ok(VideoMode::KmsDrmConsole),
        other => Err(format!("unknown video mode: {other}")),
    }
}

fn prompt_launch_options<R: BufRead, W: Write>(
    writer: &mut W,
    reader: &mut R,
    capture_mode: bool,
) -> Result<LaunchOptions, Box<dyn std::error::Error>> {
    let state_mode = prompt_state_mode(writer, reader)?;
    let state_file = prompt_path(
        writer,
        reader,
        "State file",
        DEFAULT_STATE_FILE,
        "Press Enter to keep the default path.",
    )?;
    let ui_scale = prompt_optional_ui_scale(writer, reader)?;
    let ui_scaling_mode = prompt_ui_scaling_mode(writer, reader)?;

    let run_mode = if capture_mode {
        let output_dir = prompt_path(
            writer,
            reader,
            "Capture dir",
            DEFAULT_CAPTURE_DIR,
            "Press Enter to keep the tracked screenshot directory.",
        )?;
        LaunchMode::Capture(UiCaptureOptions { output_dir })
    } else {
        let video_mode = prompt_video_mode(writer, reader)?;
        LaunchMode::Interactive(RunOptions {
            video_mode,
            session_listen: None,
        })
    };

    Ok(LaunchOptions {
        run_mode,
        state_mode,
        state_file,
        ui_scale,
        ui_scaling_mode,
        theme_preset: None,
        ui_density_preset: None,
    })
}

fn print_equivalent_command<W: Write>(writer: &mut W, options: &LaunchOptions) -> io::Result<()> {
    let mut parts = vec!["cargo run --".to_owned()];
    parts.extend(launch_command_args(options));
    writeln!(writer)?;
    writeln!(writer, "Equivalent command:")?;
    writeln!(writer, "  {}", parts.join(" "))?;
    writeln!(writer)?;
    Ok(())
}

fn prompt_state_mode<R: BufRead, W: Write>(
    writer: &mut W,
    reader: &mut R,
) -> Result<StateMode, Box<dyn std::error::Error>> {
    match prompt_menu(
        writer,
        reader,
        "State mode",
        &["persisted", "demo", "empty"],
        0,
    )? {
        0 => Ok(StateMode::Persisted),
        1 => Ok(StateMode::Demo),
        2 => Ok(StateMode::Empty),
        _ => unreachable!(),
    }
}

fn prompt_video_mode<R: BufRead, W: Write>(
    writer: &mut W,
    reader: &mut R,
) -> Result<VideoMode, Box<dyn std::error::Error>> {
    match prompt_menu(
        writer,
        reader,
        "Video mode",
        &["windowed", "fullscreen", "kmsdrm-console"],
        0,
    )? {
        0 => Ok(VideoMode::Windowed),
        1 => Ok(VideoMode::Fullscreen),
        2 => Ok(VideoMode::KmsDrmConsole),
        _ => unreachable!(),
    }
}

fn prompt_optional_ui_scale<R: BufRead, W: Write>(
    writer: &mut W,
    reader: &mut R,
) -> Result<Option<f32>, Box<dyn std::error::Error>> {
    loop {
        let value = prompt_line(
            writer,
            reader,
            "UI scale",
            "Press Enter to use the detected display scale.",
        )?;
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }
        match trimmed.parse::<f32>() {
            Ok(parsed) if parsed >= 1.0 => return Ok(Some(parsed)),
            _ => writeln!(
                writer,
                "Enter a numeric scale >= 1.0, or press Enter to skip."
            )?,
        }
    }
}

fn prompt_ui_scaling_mode<R: BufRead, W: Write>(
    writer: &mut W,
    reader: &mut R,
) -> Result<UiScalingMode, Box<dyn std::error::Error>> {
    match prompt_menu(
        writer,
        reader,
        "UI scaling mode",
        &["auto", "nearest", "linear"],
        0,
    )? {
        0 => Ok(UiScalingMode::Auto),
        1 => Ok(UiScalingMode::Nearest),
        2 => Ok(UiScalingMode::Linear),
        _ => unreachable!(),
    }
}

fn prompt_path<R: BufRead, W: Write>(
    writer: &mut W,
    reader: &mut R,
    label: &str,
    default: &str,
    hint: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    writeln!(writer, "{hint}")?;
    let value = prompt_line(writer, reader, label, &format!("Default: {default}"))?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Ok(PathBuf::from(default))
    } else {
        Ok(PathBuf::from(trimmed))
    }
}

fn prompt_menu<R: BufRead, W: Write>(
    writer: &mut W,
    reader: &mut R,
    title: &str,
    options: &[&str],
    default_index: usize,
) -> Result<usize, Box<dyn std::error::Error>> {
    loop {
        writeln!(writer, "{title}:")?;
        for (index, option) in options.iter().enumerate() {
            writeln!(writer, "  {}. {}", index + 1, option)?;
        }
        let value = prompt_line(
            writer,
            reader,
            "Choice",
            &format!("Press Enter for {}", default_index + 1),
        )?;
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Ok(default_index);
        }
        if let Ok(parsed) = trimmed.parse::<usize>() {
            if (1..=options.len()).contains(&parsed) {
                return Ok(parsed - 1);
            }
        }
        writeln!(writer, "Enter a number from 1 to {}.", options.len())?;
    }
}

fn prompt_line<R: BufRead, W: Write>(
    writer: &mut W,
    reader: &mut R,
    label: &str,
    hint: &str,
) -> io::Result<String> {
    write!(writer, "{label} [{hint}]: ")?;
    writer.flush()?;
    let mut buffer = String::new();
    reader.read_line(&mut buffer)?;
    Ok(buffer)
}

fn state_mode_label(state_mode: StateMode) -> &'static str {
    match state_mode {
        StateMode::Persisted => "persisted",
        StateMode::Demo => "demo",
        StateMode::Empty => "empty",
    }
}

fn video_mode_label(video_mode: VideoMode) -> &'static str {
    match video_mode {
        VideoMode::Windowed => "windowed",
        VideoMode::Fullscreen => "fullscreen",
        VideoMode::KmsDrmConsole => "kmsdrm-console",
    }
}

fn parse_ui_scaling_mode(value: &str) -> Result<UiScalingMode, String> {
    match value {
        "auto" => Ok(UiScalingMode::Auto),
        "nearest" => Ok(UiScalingMode::Nearest),
        "linear" => Ok(UiScalingMode::Linear),
        other => Err(format!("unknown ui scaling mode: {other}")),
    }
}

fn ui_scaling_mode_label(mode: UiScalingMode) -> &'static str {
    match mode {
        UiScalingMode::Auto => "auto",
        UiScalingMode::Nearest => "nearest",
        UiScalingMode::Linear => "linear",
    }
}

fn parse_state_mode(value: &str) -> Result<StateMode, String> {
    match value {
        "persisted" => Ok(StateMode::Persisted),
        "demo" => Ok(StateMode::Demo),
        "empty" => Ok(StateMode::Empty),
        other => Err(format!("unknown state mode: {other}")),
    }
}

fn parse_theme_preset(value: &str) -> Result<ThemePreset, String> {
    ThemePreset::from_name(value).ok_or_else(|| format!("unknown theme: {value}"))
}

fn parse_ui_density_preset(value: &str) -> Result<UiDensityPreset, String> {
    UiDensityPreset::from_name(value).ok_or_else(|| format!("unknown ui density: {value}"))
}

#[cfg(test)]
mod tests {
    use super::{AppCommand, LaunchMode, StateMode, parse_app_command_from};
    use crate::app::{UiScalingMode, VideoMode};
    use crate::theme::ThemePreset;
    use crate::ui_density::UiDensityPreset;
    use std::path::PathBuf;

    #[test]
    fn default_invocation_launches_persisted_interactive_mode() {
        let command = parse_app_command_from(Vec::<String>::new()).expect("parse command");
        let AppCommand::Launch(options) = command else {
            panic!("expected launch command");
        };
        assert_eq!(options.state_mode, StateMode::Persisted);
        assert_eq!(
            options.state_file,
            PathBuf::from("artifacts/state/last-run.json")
        );
        match options.run_mode {
            LaunchMode::Interactive(run_options) => {
                assert_eq!(run_options.video_mode, VideoMode::Windowed);
                assert_eq!(run_options.session_listen, None);
            }
            LaunchMode::Capture(_) => panic!("expected interactive mode"),
        }
    }

    #[test]
    fn run_subcommand_accepts_video_mode() {
        let command = parse_app_command_from(vec![
            "run".to_owned(),
            "--state-mode".to_owned(),
            "demo".to_owned(),
            "--video-mode".to_owned(),
            "kmsdrm-console".to_owned(),
        ])
        .expect("parse command");
        let AppCommand::Launch(options) = command else {
            panic!("expected launch command");
        };
        assert_eq!(options.state_mode, StateMode::Demo);
        match options.run_mode {
            LaunchMode::Interactive(run_options) => {
                assert_eq!(run_options.video_mode, VideoMode::KmsDrmConsole);
                assert_eq!(run_options.session_listen, None);
            }
            LaunchMode::Capture(_) => panic!("expected interactive mode"),
        }
    }

    #[test]
    fn run_subcommand_accepts_session_listen() {
        let command = parse_app_command_from(vec![
            "run".to_owned(),
            "--listen".to_owned(),
            "0.0.0.0:8787".to_owned(),
        ])
        .expect("parse command");
        let AppCommand::Launch(options) = command else {
            panic!("expected launch command");
        };
        match options.run_mode {
            LaunchMode::Interactive(run_options) => {
                assert_eq!(run_options.session_listen, Some("0.0.0.0:8787".to_owned()));
            }
            LaunchMode::Capture(_) => panic!("expected interactive mode"),
        }
    }

    #[test]
    fn capture_subcommand_defaults_to_screenshot_dir() {
        let command = parse_app_command_from(vec![
            "capture-ui".to_owned(),
            "--state-mode".to_owned(),
            "demo".to_owned(),
        ])
        .expect("parse command");
        let AppCommand::Launch(options) = command else {
            panic!("expected launch command");
        };
        assert_eq!(options.state_mode, StateMode::Demo);
        match options.run_mode {
            LaunchMode::Capture(capture) => {
                assert_eq!(capture.output_dir, PathBuf::from("artifacts/screenshots"));
            }
            LaunchMode::Interactive(_) => panic!("expected capture mode"),
        }
    }

    #[test]
    fn commands_subcommand_prints_commands_instead_of_launching() {
        let command = parse_app_command_from(vec!["commands".to_owned()]).expect("parse command");
        assert_eq!(command, AppCommand::PrintCommands);
    }

    #[test]
    fn legacy_flag_only_invocation_still_works() {
        let command = parse_app_command_from(vec!["--state-mode".to_owned(), "empty".to_owned()])
            .expect("parse command");
        let AppCommand::Launch(options) = command else {
            panic!("expected launch command");
        };
        assert_eq!(options.state_mode, StateMode::Empty);
    }

    #[test]
    fn capture_ui_rejects_video_mode() {
        let error = parse_app_command_from(vec![
            "capture-ui".to_owned(),
            "--video-mode".to_owned(),
            "fullscreen".to_owned(),
        ])
        .expect_err("capture-ui should reject video mode");
        assert_eq!(error, "--video-mode is only valid with the run command");
    }

    #[test]
    fn run_subcommand_accepts_ui_scaling_mode() {
        let command = parse_app_command_from(vec![
            "run".to_owned(),
            "--ui-scaling".to_owned(),
            "linear".to_owned(),
        ])
        .expect("parse command");
        let AppCommand::Launch(options) = command else {
            panic!("expected launch command");
        };
        assert_eq!(options.ui_scaling_mode, UiScalingMode::Linear);
    }

    #[test]
    fn ui_scaling_defaults_to_auto() {
        let command = parse_app_command_from(vec!["run".to_owned()]).expect("parse command");
        let AppCommand::Launch(options) = command else {
            panic!("expected launch command");
        };
        assert_eq!(options.ui_scaling_mode, UiScalingMode::Auto);
    }

    #[test]
    fn run_subcommand_accepts_theme() {
        let command = parse_app_command_from(vec![
            "run".to_owned(),
            "--theme".to_owned(),
            "high-contrast-light".to_owned(),
        ])
        .expect("parse command");
        let AppCommand::Launch(options) = command else {
            panic!("expected launch command");
        };
        assert_eq!(options.theme_preset, Some(ThemePreset::HighContrastLight));
    }

    fn host_session_requires_listen_addr() {
        let error =
            parse_app_command_from(vec!["host-session".to_owned()]).expect_err("expected error");
        assert_eq!(error, "--listen is required");
    }

    #[test]
    fn thin_client_requires_connect_addr() {
        let error =
            parse_app_command_from(vec!["thin-client".to_owned()]).expect_err("expected error");
        assert_eq!(error, "--connect is required");
    }

    #[test]
    fn capture_subcommand_accepts_ui_density() {
        let command = parse_app_command_from(vec![
            "capture-ui".to_owned(),
            "--ui-density".to_owned(),
            "touch".to_owned(),
        ])
        .expect("parse command");
        let AppCommand::Launch(options) = command else {
            panic!("expected launch command");
        };
        assert_eq!(options.ui_density_preset, Some(UiDensityPreset::Touch));
    }
}
