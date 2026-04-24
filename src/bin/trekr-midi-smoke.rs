use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct Config {
    list_only: bool,
    input_name: Option<String>,
    output_name: Option<String>,
    note: u8,
    velocity: u8,
    channel: u8,
    hold_ms: u64,
}

fn main() {
    let config = match parse_args() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            print_usage();
            std::process::exit(2);
        }
    };

    if let Err(error) = run(&config) {
        eprintln!("native MIDI smoke failed: {error}");
        std::process::exit(1);
    }
}

fn run(config: &Config) -> Result<(), String> {
    let mut midi_in =
        MidiInput::new("trekr-midi-smoke-input").map_err(|error| error.to_string())?;
    midi_in.ignore(Ignore::None);
    let midi_out = MidiOutput::new("trekr-midi-smoke-output").map_err(|error| error.to_string())?;

    let input_ports = midi_in.ports();
    let output_ports = midi_out.ports();
    let input_names = port_names(&midi_in, &input_ports)?;
    let output_names = port_names(&midi_out, &output_ports)?;

    println!("native midi smoke");
    println!("  inputs ({})", input_names.len());
    for name in &input_names {
        println!("    in:  {name}");
    }
    println!("  outputs ({})", output_names.len());
    for name in &output_names {
        println!("    out: {name}");
    }

    if config.list_only {
        println!("list-only run completed");
        return Ok(());
    }

    let _input_connection = match config.input_name.as_deref() {
        Some(target_name) => Some(connect_input_by_name(midi_in, target_name)?),
        None => None,
    };

    let mut output_connection = match config.output_name.as_deref() {
        Some(target_name) => Some(connect_output_by_name(midi_out, target_name)?),
        None => None,
    };

    if let Some(connection) = output_connection.as_mut() {
        let status_on = status_byte(0x90, config.channel);
        let status_off = status_byte(0x80, config.channel);
        connection
            .send(&[status_on, config.note, config.velocity])
            .map_err(|error| format!("failed sending note-on: {error}"))?;
        println!(
            "sent note-on channel={} note={} velocity={}",
            config.channel, config.note, config.velocity
        );
        thread::sleep(Duration::from_millis(config.hold_ms.max(1)));
        connection
            .send(&[status_off, config.note, 0])
            .map_err(|error| format!("failed sending note-off: {error}"))?;
        println!(
            "sent note-off channel={} note={}",
            config.channel, config.note
        );
    }

    if config.input_name.is_some() || config.output_name.is_some() {
        println!("connection run completed");
    }
    Ok(())
}

fn connect_input_by_name(
    midi_in: MidiInput,
    target_name: &str,
) -> Result<MidiInputConnection<()>, String> {
    let port = midi_in
        .ports()
        .into_iter()
        .find(|port| midi_in.port_name(port).ok().as_deref() == Some(target_name))
        .ok_or_else(|| format!("MIDI input port '{target_name}' not found"))?;

    midi_in
        .connect(
            &port,
            "trekr-midi-smoke-input-connection",
            move |timestamp, message, _state| {
                println!(
                    "received MIDI input timestamp={} bytes={:?}",
                    timestamp, message
                );
            },
            (),
        )
        .map_err(|error| error.to_string())
}

fn connect_output_by_name(
    midi_out: MidiOutput,
    target_name: &str,
) -> Result<MidiOutputConnection, String> {
    let port = midi_out
        .ports()
        .into_iter()
        .find(|port| midi_out.port_name(port).ok().as_deref() == Some(target_name))
        .ok_or_else(|| format!("MIDI output port '{target_name}' not found"))?;

    midi_out
        .connect(&port, "trekr-midi-smoke-output-connection")
        .map_err(|error| error.to_string())
}

fn port_names<P: Clone>(midi_io: &impl PortName<P>, ports: &[P]) -> Result<Vec<String>, String> {
    ports
        .iter()
        .map(|port| midi_io.port_name(port).map_err(|error| error.to_string()))
        .collect()
}

trait PortName<P> {
    fn port_name(&self, port: &P) -> Result<String, midir::PortInfoError>;
}

impl PortName<midir::MidiInputPort> for MidiInput {
    fn port_name(&self, port: &midir::MidiInputPort) -> Result<String, midir::PortInfoError> {
        MidiInput::port_name(self, port)
    }
}

impl PortName<midir::MidiOutputPort> for MidiOutput {
    fn port_name(&self, port: &midir::MidiOutputPort) -> Result<String, midir::PortInfoError> {
        MidiOutput::port_name(self, port)
    }
}

fn status_byte(base: u8, channel: u8) -> u8 {
    base | channel.saturating_sub(1).min(15)
}

fn parse_args() -> Result<Config, String> {
    let mut config = Config {
        list_only: false,
        input_name: None,
        output_name: None,
        note: 60,
        velocity: 100,
        channel: 1,
        hold_ms: 100,
    };

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--list-only" => config.list_only = true,
            "--input-name" => {
                config.input_name = Some(
                    args.next()
                        .ok_or_else(|| "missing value for --input-name".to_string())?,
                );
            }
            "--output-name" => {
                config.output_name = Some(
                    args.next()
                        .ok_or_else(|| "missing value for --output-name".to_string())?,
                );
            }
            "--note" => {
                let raw = args
                    .next()
                    .ok_or_else(|| "missing value for --note".to_string())?;
                config.note = raw
                    .parse::<u8>()
                    .map_err(|_| format!("invalid note '{raw}'"))?;
            }
            "--velocity" => {
                let raw = args
                    .next()
                    .ok_or_else(|| "missing value for --velocity".to_string())?;
                config.velocity = raw
                    .parse::<u8>()
                    .map_err(|_| format!("invalid velocity '{raw}'"))?;
            }
            "--channel" => {
                let raw = args
                    .next()
                    .ok_or_else(|| "missing value for --channel".to_string())?;
                config.channel = raw
                    .parse::<u8>()
                    .map_err(|_| format!("invalid channel '{raw}'"))?;
            }
            "--hold-ms" => {
                let raw = args
                    .next()
                    .ok_or_else(|| "missing value for --hold-ms".to_string())?;
                config.hold_ms = raw
                    .parse::<u64>()
                    .map_err(|_| format!("invalid hold-ms '{raw}'"))?;
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    if !config.list_only && config.input_name.is_none() && config.output_name.is_none() {
        return Err(
            "provide --list-only or at least one of --input-name/--output-name".to_string(),
        );
    }

    Ok(config)
}

fn print_usage() {
    println!("Native MIDI smoke utility");
    println!("Usage:");
    println!("  cargo run --bin trekr-midi-smoke -- --list-only");
    println!("  cargo run --bin trekr-midi-smoke -- --output-name <name> [options]");
    println!(
        "  cargo run --bin trekr-midi-smoke -- --input-name <name> [--output-name <name>] [options]"
    );
    println!("Options:");
    println!("  --list-only                 Enumerate native MIDI ports and exit");
    println!("  --input-name <name>         Open a native MIDI input by exact port name");
    println!("  --output-name <name>        Open a native MIDI output by exact port name");
    println!("  --note <0-127>              Default: 60");
    println!("  --velocity <0-127>          Default: 100");
    println!("  --channel <1-16>            Default: 1");
    println!("  --hold-ms <ms>              Default: 100");
}
