use crate::app::{App, CapturePadding, RunOptions, UiCaptureOptions, UiScalingMode, VideoMode};
use crate::state;
use crate::theme::ThemePreset;
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum LaunchMode {
    Interactive(RunOptions),
    Capture(UiCaptureOptions),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppCommand {
    Launch(LaunchOptions),
    PrintHelp,
    PrintCommands,
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

const SUGGESTED_COMMANDS: [SuggestedCommand; 6] = [
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
        }
    }
}

pub fn execute_app_command(command: AppCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        AppCommand::Launch(options) => launch(options),
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
    let mut app = match options.state_mode {
        StateMode::Persisted => {
            if options.state_file.exists() {
                match state::load(&options.state_file) {
                    Ok(state) => App::from_persisted_state(state),
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
    println!("{}", app.bootstrap_summary());
    match options.run_mode {
        LaunchMode::Interactive(run_options) => {
            let result = app.run_with_options(run_options);
            if result.is_ok() && options.state_mode == StateMode::Persisted {
                state::save(&options.state_file, &app.persisted_state())?;
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
        "  --video-mode <windowed|fullscreen|kmsdrm-console>   run only"
    )?;
    writeln!(
        writer,
        "  --capture-dir <path>          capture-ui only, default: {DEFAULT_CAPTURE_DIR}"
    )?;
    writeln!(
        writer,
        "  --script <path>               capture-ui only, pre-capture interaction script"
    )?;
    writeln!(
        writer,
        "  --sequence <path>             capture-ui only, emit sequence-###.png per script step"
    )?;
    writeln!(
        writer,
        "  --capture-region <id>         capture-ui only, crop to a named UI region"
    )?;
    writeln!(
        writer,
        "  --capture-rect x,y,w,h        capture-ui only, crop to explicit rectangle"
    )?;
    writeln!(
        writer,
        "  --capture-padding <px|l,t,r,b> capture-ui only, expand crop region with padding"
    )?;
    writeln!(
        writer,
        "  --annotate <path>             capture-ui only, optional overlay annotations JSON"
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
        }
        LaunchMode::Capture(capture) => {
            args.push("capture-ui".to_owned());
            if capture.output_dir != PathBuf::from(DEFAULT_CAPTURE_DIR) {
                args.push("--capture-dir".to_owned());
                args.push(capture.output_dir.display().to_string());
            }
            if let Some(path) = &capture.script_path {
                args.push("--script".to_owned());
                args.push(path.display().to_string());
            }
            if let Some(path) = &capture.sequence_path {
                args.push("--sequence".to_owned());
                args.push(path.display().to_string());
            }
            if let Some(region) = &capture.capture_region {
                args.push("--capture-region".to_owned());
                args.push(region.clone());
            }
            if let Some(rect) = &capture.capture_rect {
                args.push("--capture-rect".to_owned());
                args.push(format!(
                    "{},{},{},{}",
                    rect.x, rect.y, rect.width, rect.height
                ));
            }
            if let Some(padding) = &capture.capture_padding {
                args.push("--capture-padding".to_owned());
                args.push(format!(
                    "{},{},{},{}",
                    padding.left, padding.top, padding.right, padding.bottom
                ));
            }
            if let Some(path) = &capture.annotation_path {
                args.push("--annotate".to_owned());
                args.push(path.display().to_string());
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

    args
}

fn parse_launch_options_from<I>(args: I, capture_mode: bool) -> Result<LaunchOptions, String>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let mut capture_dir = capture_mode.then(|| PathBuf::from(DEFAULT_CAPTURE_DIR));
    let mut capture_script = None;
    let mut capture_region = None;
    let mut capture_rect = None;
    let mut capture_padding = None;
    let mut annotation_path = None;
    let mut sequence_path = None;
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
            "--script" => {
                if !capture_mode && capture_dir.is_none() {
                    return Err("--script is only valid with the capture-ui command".to_owned());
                }
                let value = args
                    .next()
                    .ok_or_else(|| "--script requires a path".to_owned())?;
                capture_script = Some(PathBuf::from(value));
            }
            "--sequence" => {
                if !capture_mode && capture_dir.is_none() {
                    return Err("--sequence is only valid with the capture-ui command".to_owned());
                }
                let value = args
                    .next()
                    .ok_or_else(|| "--sequence requires a path".to_owned())?;
                sequence_path = Some(PathBuf::from(value));
            }
            "--capture-region" => {
                if !capture_mode && capture_dir.is_none() {
                    return Err(
                        "--capture-region is only valid with the capture-ui command".to_owned()
                    );
                }
                let value = args
                    .next()
                    .ok_or_else(|| "--capture-region requires an id".to_owned())?;
                capture_region = Some(value);
            }
            "--capture-rect" => {
                if !capture_mode && capture_dir.is_none() {
                    return Err(
                        "--capture-rect is only valid with the capture-ui command".to_owned()
                    );
                }
                let value = args
                    .next()
                    .ok_or_else(|| "--capture-rect requires x,y,w,h".to_owned())?;
                capture_rect = Some(parse_capture_rect(&value)?);
            }
            "--capture-padding" => {
                if !capture_mode && capture_dir.is_none() {
                    return Err(
                        "--capture-padding is only valid with the capture-ui command".to_owned(),
                    );
                }
                let value = args.next().ok_or_else(|| {
                    "--capture-padding requires <px> or left,top,right,bottom".to_owned()
                })?;
                capture_padding = Some(parse_capture_padding(&value)?);
            }
            "--annotate" => {
                if !capture_mode && capture_dir.is_none() {
                    return Err("--annotate is only valid with the capture-ui command".to_owned());
                }
                let value = args
                    .next()
                    .ok_or_else(|| "--annotate requires a path".to_owned())?;
                annotation_path = Some(PathBuf::from(value));
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
            "--video-mode" => {
                let value = args.next().ok_or_else(|| {
                    "--video-mode requires windowed|fullscreen|kmsdrm-console".to_owned()
                })?;
                if capture_mode {
                    return Err("--video-mode is only valid with the run command".to_owned());
                }
                run_options.video_mode = parse_video_mode(&value)?;
            }
            "--help" | "-h" => return Err("use `help` to print the full CLI reference".to_owned()),
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    options.run_mode = match capture_dir {
        Some(output_dir) => LaunchMode::Capture(UiCaptureOptions {
            output_dir,
            state_mode: state_mode_label(options.state_mode).to_owned(),
            script_path: capture_script,
            capture_region,
            capture_rect,
            capture_padding,
            annotation_path,
            sequence_path,
        }),
        None => LaunchMode::Interactive(run_options),
    };

    Ok(options)
}

fn parse_video_mode(value: &str) -> Result<VideoMode, String> {
    match value {
        "windowed" => Ok(VideoMode::Windowed),
        "fullscreen" => Ok(VideoMode::Fullscreen),
        "kmsdrm-console" | "kmsdrm" => Ok(VideoMode::KmsDrmConsole),
        other => Err(format!("unknown video mode: {other}")),
    }
}

fn parse_capture_rect(value: &str) -> Result<crate::app::CaptureRect, String> {
    let parts = value.split(',').collect::<Vec<_>>();
    if parts.len() != 4 {
        return Err("--capture-rect requires x,y,w,h".to_owned());
    }
    let x = parts[0]
        .parse::<u32>()
        .map_err(|_| format!("invalid capture rect x value: {}", parts[0]))?;
    let y = parts[1]
        .parse::<u32>()
        .map_err(|_| format!("invalid capture rect y value: {}", parts[1]))?;
    let width = parts[2]
        .parse::<u32>()
        .map_err(|_| format!("invalid capture rect width value: {}", parts[2]))?;
    let height = parts[3]
        .parse::<u32>()
        .map_err(|_| format!("invalid capture rect height value: {}", parts[3]))?;
    if width == 0 || height == 0 {
        return Err("capture rect width and height must be greater than zero".to_owned());
    }
    Ok(crate::app::CaptureRect {
        x,
        y,
        width,
        height,
    })
}

fn parse_capture_padding(value: &str) -> Result<CapturePadding, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("--capture-padding requires <px> or left,top,right,bottom".to_owned());
    }
    let parts = trimmed
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    match parts.len() {
        1 => {
            let uniform = parts[0]
                .parse::<u32>()
                .map_err(|_| format!("invalid capture padding value: {}", parts[0]))?;
            Ok(CapturePadding {
                left: uniform,
                top: uniform,
                right: uniform,
                bottom: uniform,
            })
        }
        4 => {
            let left = parts[0]
                .parse::<u32>()
                .map_err(|_| format!("invalid capture padding left value: {}", parts[0]))?;
            let top = parts[1]
                .parse::<u32>()
                .map_err(|_| format!("invalid capture padding top value: {}", parts[1]))?;
            let right = parts[2]
                .parse::<u32>()
                .map_err(|_| format!("invalid capture padding right value: {}", parts[2]))?;
            let bottom = parts[3]
                .parse::<u32>()
                .map_err(|_| format!("invalid capture padding bottom value: {}", parts[3]))?;
            Ok(CapturePadding {
                left,
                top,
                right,
                bottom,
            })
        }
        _ => Err("--capture-padding requires <px> or left,top,right,bottom".to_owned()),
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

    let run_mode = if capture_mode {
        let output_dir = prompt_path(
            writer,
            reader,
            "Capture dir",
            DEFAULT_CAPTURE_DIR,
            "Press Enter to keep the tracked screenshot directory.",
        )?;
        LaunchMode::Capture(UiCaptureOptions {
            output_dir,
            state_mode: state_mode_label(state_mode).to_owned(),
            script_path: None,
            capture_region: None,
            capture_rect: None,
            capture_padding: None,
            annotation_path: None,
            sequence_path: None,
        })
    } else {
        let video_mode = prompt_video_mode(writer, reader)?;
        LaunchMode::Interactive(RunOptions { video_mode })
    };

    Ok(LaunchOptions {
        run_mode,
        state_mode,
        state_file,
        ui_scale,
        ui_scaling_mode,
        theme_preset: None,
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

#[cfg(test)]
mod tests {
    use super::{AppCommand, LaunchMode, StateMode, parse_app_command_from};
    use crate::app::{UiScalingMode, VideoMode};
    use crate::theme::ThemePreset;
    use crate::app::CapturePadding;
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
                assert_eq!(capture.state_mode, "demo");
                assert!(capture.script_path.is_none());
                assert!(capture.sequence_path.is_none());
                assert!(capture.capture_region.is_none());
                assert!(capture.capture_rect.is_none());
                assert!(capture.capture_padding.is_none());
                assert!(capture.annotation_path.is_none());
            }
            LaunchMode::Interactive(_) => panic!("expected capture mode"),
        }
    }

    #[test]
    fn capture_ui_accepts_script_region_rect_and_annotations() {
        let command = parse_app_command_from(vec![
            "capture-ui".to_owned(),
            "--script".to_owned(),
            "artifacts/capture-scripts/direct-map.json".to_owned(),
            "--sequence".to_owned(),
            "artifacts/capture-scripts/tutorial-sequence.json".to_owned(),
            "--capture-region".to_owned(),
            "timeline_transport".to_owned(),
            "--capture-rect".to_owned(),
            "12,24,320,120".to_owned(),
            "--capture-padding".to_owned(),
            "16".to_owned(),
            "--annotate".to_owned(),
            "artifacts/capture-scripts/overlay.json".to_owned(),
        ])
        .expect("parse command");
        let AppCommand::Launch(options) = command else {
            panic!("expected launch command");
        };
        match options.run_mode {
            LaunchMode::Capture(capture) => {
                assert_eq!(
                    capture.script_path,
                    Some(PathBuf::from("artifacts/capture-scripts/direct-map.json"))
                );
                assert_eq!(
                    capture.sequence_path,
                    Some(PathBuf::from(
                        "artifacts/capture-scripts/tutorial-sequence.json"
                    ))
                );
                assert_eq!(
                    capture.capture_region,
                    Some("timeline_transport".to_owned())
                );
                assert_eq!(capture.capture_rect.expect("capture rect").width, 320);
                assert_eq!(
                    capture.capture_padding,
                    Some(CapturePadding {
                        left: 16,
                        top: 16,
                        right: 16,
                        bottom: 16
                    })
                );
                assert_eq!(
                    capture.annotation_path,
                    Some(PathBuf::from("artifacts/capture-scripts/overlay.json"))
                );
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
    fn legacy_capture_flag_accepts_new_capture_options() {
        let command = parse_app_command_from(vec![
            "--capture-ui".to_owned(),
            "--script".to_owned(),
            "artifacts/capture-scripts/direct-map.json".to_owned(),
        ])
        .expect("parse command");
        let AppCommand::Launch(options) = command else {
            panic!("expected launch command");
        };
        match options.run_mode {
            LaunchMode::Capture(capture) => {
                assert_eq!(
                    capture.script_path,
                    Some(PathBuf::from("artifacts/capture-scripts/direct-map.json"))
                );
            }
            LaunchMode::Interactive(_) => panic!("expected capture mode"),
        }
    }

    #[test]
    fn capture_ui_accepts_per_edge_padding() {
        let command = parse_app_command_from(vec![
            "capture-ui".to_owned(),
            "--capture-padding".to_owned(),
            "4,8,12,16".to_owned(),
        ])
        .expect("parse command");
        let AppCommand::Launch(options) = command else {
            panic!("expected launch command");
        };
        match options.run_mode {
            LaunchMode::Capture(capture) => {
                assert_eq!(
                    capture.capture_padding,
                    Some(CapturePadding {
                        left: 4,
                        top: 8,
                        right: 12,
                        bottom: 16
                    })
                );
            }
            LaunchMode::Interactive(_) => panic!("expected capture mode"),
        }
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
}
