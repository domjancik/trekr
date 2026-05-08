use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use std::cmp::Ordering;
use std::io::{self, Write};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Config {
    list_only: bool,
    input_name: Option<String>,
    output_name: Option<String>,
    note: u8,
    velocity: u8,
    channel: u8,
    count: u32,
    warmup_count: u32,
    interval_ms: u64,
    note_length_ms: u64,
    timeout_ms: u64,
}

#[derive(Debug, Clone)]
struct LoopbackSample {
    pitch: u8,
    latency: Duration,
}

#[derive(Debug, Clone, PartialEq)]
struct SummaryStats {
    count: usize,
    min_ms: f64,
    max_ms: f64,
    mean_ms: f64,
    p50_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
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
        eprintln!("midi loopback latency test failed: {error}");
        std::process::exit(1);
    }
}

fn run(config: &Config) -> Result<(), String> {
    let mut midi_in =
        MidiInput::new("trekr-midi-loopback-input").map_err(|error| error.to_string())?;
    midi_in.ignore(Ignore::None);
    let midi_out =
        MidiOutput::new("trekr-midi-loopback-output").map_err(|error| error.to_string())?;

    let input_ports = midi_in.ports();
    let output_ports = midi_out.ports();
    let input_names = port_names(&midi_in, &input_ports)?;
    let output_names = port_names(&midi_out, &output_ports)?;

    println!("trekr midi loopback latency");
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

    let input_name = resolve_port_name(config.input_name.as_deref(), &input_names, "input")?;
    let output_name = resolve_port_name(config.output_name.as_deref(), &output_names, "output")?;

    let (input_receiver, _input_connection) = connect_input_by_name(midi_in, &input_name)?;
    let mut output_connection = connect_output_by_name(midi_out, &output_name)?;

    println!(
        "starting loopback test channel={} base_note={} velocity={} warmup={} count={} interval_ms={} timeout_ms={}",
        config.channel,
        config.note,
        config.velocity,
        config.warmup_count,
        config.count,
        config.interval_ms,
        config.timeout_ms
    );

    for index in 0..config.warmup_count {
        let pitch = cycle_pitch(config.note, index)?;
        let _ = send_and_measure(
            config,
            &mut output_connection,
            &input_receiver,
            pitch,
            false,
        )?;
    }

    let mut samples = Vec::with_capacity(config.count as usize);
    for index in 0..config.count {
        let pitch = cycle_pitch(config.note, config.warmup_count + index)?;
        let sample =
            send_and_measure(config, &mut output_connection, &input_receiver, pitch, true)?;
        samples.push(sample);
    }

    let stats = summarize(&samples);
    println!("completed loopback test");
    println!("  samples: {}", stats.count);
    println!("  min_ms: {:.3}", stats.min_ms);
    println!("  p50_ms: {:.3}", stats.p50_ms);
    println!("  p95_ms: {:.3}", stats.p95_ms);
    println!("  p99_ms: {:.3}", stats.p99_ms);
    println!("  max_ms: {:.3}", stats.max_ms);
    println!("  mean_ms: {:.3}", stats.mean_ms);

    Ok(())
}

fn resolve_port_name(
    configured_name: Option<&str>,
    available_names: &[String],
    port_kind: &str,
) -> Result<String, String> {
    if let Some(name) = configured_name {
        if available_names.iter().any(|available| available == name) {
            return Ok(name.to_string());
        }
        return Err(format!(
            "requested MIDI {} port '{}' was not found",
            port_kind, name
        ));
    }

    prompt_for_port_selection(available_names, port_kind)
}

fn prompt_for_port_selection(
    available_names: &[String],
    port_kind: &str,
) -> Result<String, String> {
    if available_names.is_empty() {
        return Err(format!(
            "no MIDI {} ports are available to select",
            port_kind
        ));
    }

    println!("select MIDI {} port:", port_kind);
    for (index, name) in available_names.iter().enumerate() {
        println!("  {}. {}", index + 1, name);
    }

    loop {
        print!(
            "enter {} port number [1-{}]: ",
            port_kind,
            available_names.len()
        );
        io::stdout()
            .flush()
            .map_err(|error| format!("failed flushing prompt: {error}"))?;

        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .map_err(|error| format!("failed reading {} selection: {error}", port_kind))?;
        let trimmed = line.trim();
        let selection = trimmed.parse::<usize>().map_err(|_| {
            format!(
                "invalid {} selection '{}': enter a number from 1 to {}",
                port_kind,
                trimmed,
                available_names.len()
            )
        });

        match selection {
            Ok(value) if (1..=available_names.len()).contains(&value) => {
                return Ok(available_names[value - 1].clone());
            }
            Ok(_) => {
                eprintln!(
                    "selection out of range for MIDI {} port: enter a number from 1 to {}",
                    port_kind,
                    available_names.len()
                );
            }
            Err(message) => {
                eprintln!("{message}");
            }
        }
    }
}

fn send_and_measure(
    config: &Config,
    output_connection: &mut MidiOutputConnection,
    input_receiver: &Receiver<LoopbackInputEvent>,
    pitch: u8,
    print_sample: bool,
) -> Result<LoopbackSample, String> {
    drain_pending(input_receiver);

    let status_on = status_byte(0x90, config.channel);
    let status_off = status_byte(0x80, config.channel);
    let sent_at = Instant::now();
    output_connection
        .send(&[status_on, pitch, config.velocity])
        .map_err(|error| format!("failed sending note-on for pitch {pitch}: {error}"))?;

    let timeout = Duration::from_millis(config.timeout_ms.max(1));
    let received = wait_for_matching_event(input_receiver, pitch, config.velocity, timeout)?;
    let sample = LoopbackSample {
        pitch,
        latency: received.received_at.saturating_duration_since(sent_at),
    };

    thread::sleep(Duration::from_millis(config.note_length_ms.max(1)));
    output_connection
        .send(&[status_off, pitch, 0])
        .map_err(|error| format!("failed sending note-off for pitch {pitch}: {error}"))?;

    if print_sample {
        println!(
            "  sample pitch={} latency_ms={:.3}",
            sample.pitch,
            duration_ms(sample.latency)
        );
    }

    thread::sleep(Duration::from_millis(config.interval_ms));
    Ok(sample)
}

fn wait_for_matching_event(
    input_receiver: &Receiver<LoopbackInputEvent>,
    pitch: u8,
    velocity: u8,
    timeout: Duration,
) -> Result<LoopbackInputEvent, String> {
    let deadline = Instant::now() + timeout;
    loop {
        let now = Instant::now();
        if now >= deadline {
            return Err(format!(
                "timed out waiting for loopback note-on pitch={} velocity={} after {} ms",
                pitch,
                velocity,
                timeout.as_millis()
            ));
        }
        let remaining = deadline.saturating_duration_since(now);
        let event = input_receiver.recv_timeout(remaining).map_err(|_| {
            format!(
                "timed out waiting for loopback note-on pitch={} velocity={} after {} ms",
                pitch,
                velocity,
                timeout.as_millis()
            )
        })?;
        if event.pitch == pitch && event.velocity == velocity {
            return Ok(event);
        }
    }
}

fn drain_pending(input_receiver: &Receiver<LoopbackInputEvent>) {
    while input_receiver.try_recv().is_ok() {}
}

fn cycle_pitch(base_note: u8, index: u32) -> Result<u8, String> {
    let offset = (index % 12) as u8;
    base_note
        .checked_add(offset)
        .filter(|pitch| *pitch <= 127)
        .ok_or_else(|| format!("base note {} plus rotating offset exceeds 127", base_note))
}

fn summarize(samples: &[LoopbackSample]) -> SummaryStats {
    let mut values = samples
        .iter()
        .map(|sample| duration_ms(sample.latency))
        .collect::<Vec<_>>();
    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
    let count = values.len();
    let min_ms = *values.first().unwrap_or(&0.0);
    let max_ms = *values.last().unwrap_or(&0.0);
    let mean_ms = if count == 0 {
        0.0
    } else {
        values.iter().sum::<f64>() / count as f64
    };
    SummaryStats {
        count,
        min_ms,
        max_ms,
        mean_ms,
        p50_ms: percentile(&values, 0.50),
        p95_ms: percentile(&values, 0.95),
        p99_ms: percentile(&values, 0.99),
    }
}

fn percentile(sorted_values: &[f64], percentile: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    let index = ((sorted_values.len() - 1) as f64 * percentile).round() as usize;
    sorted_values[index.min(sorted_values.len() - 1)]
}

fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

#[derive(Debug, Clone)]
struct LoopbackInputEvent {
    pitch: u8,
    velocity: u8,
    received_at: Instant,
}

fn connect_input_by_name(
    midi_in: MidiInput,
    target_name: &str,
) -> Result<(Receiver<LoopbackInputEvent>, MidiInputConnection<()>), String> {
    let (sender, receiver) = mpsc::channel();
    let port = midi_in
        .ports()
        .into_iter()
        .find(|port| midi_in.port_name(port).ok().as_deref() == Some(target_name))
        .ok_or_else(|| format!("MIDI input port '{target_name}' not found"))?;

    let connection = midi_in
        .connect(
            &port,
            "trekr-midi-loopback-input-connection",
            move |_timestamp, message, _state| {
                if let Some(event) = parse_note_on(message) {
                    let _ = sender.send(LoopbackInputEvent {
                        pitch: event.0,
                        velocity: event.1,
                        received_at: Instant::now(),
                    });
                }
            },
            (),
        )
        .map_err(|error| error.to_string())?;

    Ok((receiver, connection))
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
        .connect(&port, "trekr-midi-loopback-output-connection")
        .map_err(|error| error.to_string())
}

fn parse_note_on(message: &[u8]) -> Option<(u8, u8)> {
    let status = *message.first()?;
    if status & 0xF0 != 0x90 {
        return None;
    }
    let pitch = *message.get(1)?;
    let velocity = *message.get(2)?;
    (velocity > 0).then_some((pitch, velocity))
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
        count: 32,
        warmup_count: 4,
        interval_ms: 40,
        note_length_ms: 10,
        timeout_ms: 500,
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
            "--note" => config.note = parse_value(&mut args, "--note")?,
            "--velocity" => config.velocity = parse_value(&mut args, "--velocity")?,
            "--channel" => config.channel = parse_value(&mut args, "--channel")?,
            "--count" => config.count = parse_value(&mut args, "--count")?,
            "--warmup-count" => config.warmup_count = parse_value(&mut args, "--warmup-count")?,
            "--interval-ms" => config.interval_ms = parse_value(&mut args, "--interval-ms")?,
            "--note-length-ms" => {
                config.note_length_ms = parse_value(&mut args, "--note-length-ms")?
            }
            "--timeout-ms" => config.timeout_ms = parse_value(&mut args, "--timeout-ms")?,
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    if !(1..=16).contains(&config.channel) {
        return Err(format!(
            "invalid channel '{}': expected 1-16",
            config.channel
        ));
    }

    Ok(config)
}

fn parse_value<T: std::str::FromStr>(
    args: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<T, String> {
    let raw = args
        .next()
        .ok_or_else(|| format!("missing value for {flag}"))?;
    raw.parse::<T>()
        .map_err(|_| format!("invalid value '{raw}' for {flag}"))
}

fn print_usage() {
    println!("Trekr MIDI loopback latency utility");
    println!("Usage:");
    println!("  cargo run --bin trekr-midi-loopback-latency -- --list-only");
    println!(
        "  cargo run --bin trekr-midi-loopback-latency -- --input-name <name> --output-name <name> [options]"
    );
    println!(
        "  cargo run --bin trekr-midi-loopback-latency -- [options]    # prompts interactively when ports are omitted"
    );
    println!("Options:");
    println!("  --list-only                 Enumerate native MIDI ports and exit");
    println!("  --input-name <name>         Open a native MIDI input by exact port name");
    println!("  --output-name <name>        Open a native MIDI output by exact port name");
    println!("  --note <0-127>              Base test note. Default: 60");
    println!("  --velocity <0-127>          Default: 100");
    println!("  --channel <1-16>            Default: 1");
    println!("  --count <n>                 Measured samples. Default: 32");
    println!("  --warmup-count <n>          Warmup samples not included in stats. Default: 4");
    println!("  --interval-ms <ms>          Delay between measured pings. Default: 40");
    println!("  --note-length-ms <ms>       Note hold before note-off. Default: 10");
    println!("  --timeout-ms <ms>           Per-sample loopback timeout. Default: 500");
}

#[cfg(test)]
mod tests {
    use super::{
        LoopbackSample, duration_ms, percentile, prompt_for_port_selection, resolve_port_name,
        summarize,
    };
    use std::time::Duration;

    #[test]
    fn resolve_port_name_accepts_existing_configured_name() {
        let ports = vec!["In A".to_string(), "In B".to_string()];
        assert_eq!(
            resolve_port_name(Some("In B"), &ports, "input").unwrap(),
            "In B".to_string()
        );
    }

    #[test]
    fn resolve_port_name_rejects_missing_configured_name() {
        let ports = vec!["In A".to_string()];
        assert!(resolve_port_name(Some("Missing"), &ports, "input").is_err());
    }

    #[test]
    fn prompt_selection_fails_without_ports() {
        assert!(prompt_for_port_selection(&[], "output").is_err());
    }

    #[test]
    fn percentile_uses_sorted_values() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&values, 0.50), 3.0);
        assert_eq!(percentile(&values, 0.95), 5.0);
    }

    #[test]
    fn summarize_reports_expected_stats() {
        let samples = vec![
            LoopbackSample {
                pitch: 60,
                latency: Duration::from_millis(1),
            },
            LoopbackSample {
                pitch: 61,
                latency: Duration::from_millis(2),
            },
            LoopbackSample {
                pitch: 62,
                latency: Duration::from_millis(5),
            },
            LoopbackSample {
                pitch: 63,
                latency: Duration::from_millis(9),
            },
        ];
        let stats = summarize(&samples);
        assert_eq!(stats.count, 4);
        assert_eq!(stats.min_ms, 1.0);
        assert_eq!(stats.max_ms, 9.0);
        assert!(stats.mean_ms >= 4.0 && stats.mean_ms <= 5.0);
        assert_eq!(stats.p50_ms, 5.0);
    }

    #[test]
    fn duration_ms_converts_subsecond_values() {
        assert!((duration_ms(Duration::from_micros(1500)) - 1.5).abs() < 0.001);
    }
}
