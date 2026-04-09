use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum LauncherCommand {
    Ui,
    Help,
    ListBranches {
        repo_url: Option<String>,
    },
    Install {
        branch: String,
        repo_url: Option<String>,
        rebuild: bool,
        allow_source_build: bool,
    },
    Run(RunLauncherOptions),
    ListInstalled,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunLauncherOptions {
    pub branch: String,
    pub project: Option<PathBuf>,
    pub state_mode: Option<String>,
    pub window_mode: Option<String>,
    pub ui_scale: Option<f32>,
    pub extra_args: Vec<String>,
}

pub fn parse_command_from<I>(args: I) -> Result<LauncherCommand, Box<dyn std::error::Error>>
where
    I: IntoIterator<Item = String>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    let Some(command) = args.first() else {
        return Ok(LauncherCommand::Ui);
    };

    match command.as_str() {
        "ui" => Ok(LauncherCommand::Ui),
        "help" | "--help" | "-h" => Ok(LauncherCommand::Help),
        "list-branches" => {
            let mut repo_url = None;
            let mut iter = args.into_iter().skip(1);
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--repo-url" => {
                        repo_url = Some(
                            iter.next()
                                .ok_or("--repo-url requires a value")?
                                .trim()
                                .to_string(),
                        );
                    }
                    other => {
                        return Err(format!("unknown argument for list-branches: {other}").into());
                    }
                }
            }
            Ok(LauncherCommand::ListBranches { repo_url })
        }
        "install" => {
            let mut branch = None;
            let mut repo_url = None;
            let mut rebuild = false;
            let mut allow_source_build = false;
            let mut iter = args.into_iter().skip(1);
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--branch" => {
                        branch = Some(
                            iter.next()
                                .ok_or("--branch requires a value")?
                                .trim()
                                .to_string(),
                        );
                    }
                    "--repo-url" => {
                        repo_url = Some(
                            iter.next()
                                .ok_or("--repo-url requires a value")?
                                .trim()
                                .to_string(),
                        );
                    }
                    "--rebuild" => rebuild = true,
                    "--allow-source-build" => allow_source_build = true,
                    other => return Err(format!("unknown argument for install: {other}").into()),
                }
            }
            let branch = branch.ok_or("install requires --branch <name>")?;
            Ok(LauncherCommand::Install {
                branch,
                repo_url,
                rebuild,
                allow_source_build,
            })
        }
        "run" => Ok(LauncherCommand::Run(parse_run_options(
            args.into_iter().skip(1).collect(),
        )?)),
        "list-installed" => Ok(LauncherCommand::ListInstalled),
        other => Err(format!("unknown command: {other}").into()),
    }
}

fn parse_run_options(args: Vec<String>) -> Result<RunLauncherOptions, Box<dyn std::error::Error>> {
    let mut branch = None;
    let mut project = None;
    let mut state_mode = None;
    let mut window_mode = None;
    let mut ui_scale = None;
    let mut extra_args = Vec::new();

    let mut iter = args.into_iter().peekable();
    while let Some(arg) = iter.next() {
        if arg == "--" {
            extra_args.extend(iter);
            break;
        }
        match arg.as_str() {
            "--branch" => {
                branch = Some(iter.next().ok_or("--branch requires a value")?);
            }
            "--project" => {
                project = Some(PathBuf::from(
                    iter.next().ok_or("--project requires a path")?,
                ));
            }
            "--state-mode" => {
                state_mode = Some(iter.next().ok_or("--state-mode requires a value")?);
            }
            "--window-mode" => {
                window_mode = Some(iter.next().ok_or("--window-mode requires a value")?);
            }
            "--ui-scale" => {
                let value = iter.next().ok_or("--ui-scale requires a value")?;
                let parsed = value
                    .parse::<f32>()
                    .map_err(|_| format!("invalid --ui-scale value: {value}"))?;
                if parsed < 1.0 {
                    return Err("--ui-scale must be at least 1.0".into());
                }
                ui_scale = Some(parsed);
            }
            other => return Err(format!("unknown argument for run: {other}").into()),
        }
    }

    let branch = branch.ok_or("run requires --branch <name>")?;
    Ok(RunLauncherOptions {
        branch,
        project,
        state_mode,
        window_mode,
        ui_scale,
        extra_args,
    })
}

pub fn print_help<W: Write>(writer: &mut W) -> io::Result<()> {
    writeln!(writer, "trekr-launcher")?;
    writeln!(writer)?;
    writeln!(
        writer,
        "usage: cargo run --bin trekr-launcher -- <command> [options]"
    )?;
    writeln!(writer)?;
    writeln!(writer, "commands:")?;
    writeln!(writer, "  ui   (default when no command is given)")?;
    writeln!(writer, "  list-branches [--repo-url <url>]")?;
    writeln!(
        writer,
        "  install --branch <name> [--repo-url <url>] [--rebuild] [--allow-source-build]"
    )?;
    writeln!(
        writer,
        "  run --branch <name> [--project <state-file>] [--window-mode <windowed|fullscreen|kmsdrm-console>] [--state-mode <persisted|demo|empty>] [--ui-scale <n>] [-- <extra trekr args>]"
    )?;
    writeln!(writer, "  list-installed")?;
    writeln!(writer, "  help")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{LauncherCommand, parse_command_from};

    #[test]
    fn parse_install_command() {
        let command = parse_command_from(vec![
            "install".to_string(),
            "--branch".to_string(),
            "feature/x".to_string(),
            "--rebuild".to_string(),
        ])
        .expect("install command");
        assert_eq!(
            command,
            LauncherCommand::Install {
                branch: "feature/x".to_string(),
                repo_url: None,
                rebuild: true,
                allow_source_build: false
            }
        );
    }

    #[test]
    fn parse_run_command_with_arguments() {
        let command = parse_command_from(vec![
            "run".to_string(),
            "--branch".to_string(),
            "main".to_string(),
            "--project".to_string(),
            "state-fixtures\\ui-looped.json".to_string(),
            "--window-mode".to_string(),
            "fullscreen".to_string(),
            "--state-mode".to_string(),
            "persisted".to_string(),
            "--ui-scale".to_string(),
            "1.5".to_string(),
            "--".to_string(),
            "--capture-ui".to_string(),
        ])
        .expect("run command");
        let LauncherCommand::Run(options) = command else {
            panic!("expected run");
        };
        assert_eq!(options.branch, "main");
        assert_eq!(options.window_mode.as_deref(), Some("fullscreen"));
        assert_eq!(options.state_mode.as_deref(), Some("persisted"));
        assert_eq!(options.ui_scale, Some(1.5));
        assert_eq!(options.extra_args, vec!["--capture-ui"]);
    }
}
