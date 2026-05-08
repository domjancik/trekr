use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Config {
    list_only: bool,
    input_name: Option<String>,
    output_name: Option<String>,
    trigger_input_name: Option<String>,
    return_input_name: Option<String>,
    note: u8,
    velocity: u8,
    channel: u8,
    count: u32,
    warmup_count: u32,
    interval_ms: u64,
    note_length_ms: u64,
    timeout_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    SelfSend,
    DeviceInitiated,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct PersistedSelections {
    run_mode: Option<PersistedRunMode>,
    input_name: Option<String>,
    output_name: Option<String>,
    trigger_input_name: Option<String>,
    return_input_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PersistedRunMode {
    SelfSend,
    DeviceInitiated,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MidiNoteOn {
    channel: u8,
    pitch: u8,
    velocity: u8,
}

#[derive(Debug, Clone)]
struct MidiObservedEvent {
    source_label: String,
    note_on: Option<MidiNoteOn>,
    message_bytes: Vec<u8>,
    received_at: Instant,
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

    let persisted = load_persisted_selections();
    match detect_run_mode(config, persisted.as_ref())? {
        RunMode::SelfSend => run_self_send(config, &input_names, &output_names, persisted.as_ref()),
        RunMode::DeviceInitiated => {
            run_device_initiated(config, &input_names, &output_names, persisted.as_ref())
        }
    }
}

fn detect_run_mode(
    config: &Config,
    persisted: Option<&PersistedSelections>,
) -> Result<RunMode, String> {
    if config.trigger_input_name.is_some() || config.return_input_name.is_some() {
        return Ok(RunMode::DeviceInitiated);
    }

    if config.input_name.is_some() || config.output_name.is_some() {
        return Ok(RunMode::SelfSend);
    }

    prompt_for_run_mode(persisted.and_then(|state| state.run_mode))
}

fn prompt_for_run_mode(default_mode: Option<PersistedRunMode>) -> Result<RunMode, String> {
    if let Some(default_mode) = default_mode {
        let label = match default_mode {
            PersistedRunMode::SelfSend => "self-send loopback",
            PersistedRunMode::DeviceInitiated => "device-initiated trigger -> output -> return",
        };
        if confirm_default(&format!("use last measurement mode ({label})"), true)? {
            return Ok(match default_mode {
                PersistedRunMode::SelfSend => RunMode::SelfSend,
                PersistedRunMode::DeviceInitiated => RunMode::DeviceInitiated,
            });
        }
    }

    println!("select measurement mode:");
    println!("  1. self-send loopback");
    println!("  2. device-initiated trigger -> output -> return");

    loop {
        print!("enter mode number [1-2]: ");
        io::stdout()
            .flush()
            .map_err(|error| format!("failed flushing mode prompt: {error}"))?;

        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .map_err(|error| format!("failed reading mode selection: {error}"))?;

        match line.trim() {
            "1" => return Ok(RunMode::SelfSend),
            "2" => return Ok(RunMode::DeviceInitiated),
            other => eprintln!("invalid mode selection '{}': enter 1 or 2", other),
        }
    }
}

fn run_self_send(
    config: &Config,
    input_names: &[String],
    output_names: &[String],
    persisted: Option<&PersistedSelections>,
) -> Result<(), String> {
    let input_name = resolve_port_name(
        config.input_name.as_deref(),
        persisted.and_then(|state| state.input_name.as_deref()),
        input_names,
        "input",
    )?;
    let output_name = resolve_port_name(
        config.output_name.as_deref(),
        persisted.and_then(|state| state.output_name.as_deref()),
        output_names,
        "output",
    )?;

    save_persisted_selections(&PersistedSelections {
        run_mode: Some(PersistedRunMode::SelfSend),
        input_name: Some(input_name.clone()),
        output_name: Some(output_name.clone()),
        trigger_input_name: None,
        return_input_name: None,
    });

    let (input_receiver, _input_connection) = connect_input_by_name(
        &input_name,
        "trekr-midi-loopback-input-connection",
        "return",
    )?;
    let mut output_connection = connect_output_by_name(&output_name)?;

    println!(
        "starting self-send loopback test channel={} base_note={} velocity={} warmup={} count={} interval_ms={} timeout_ms={}",
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
        let _ = send_and_measure_self_send(
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
        let sample = send_and_measure_self_send(
            config,
            &mut output_connection,
            &input_receiver,
            pitch,
            true,
        )?;
        samples.push(sample);
    }

    print_summary(&samples);
    Ok(())
}

fn run_device_initiated(
    config: &Config,
    input_names: &[String],
    output_names: &[String],
    persisted: Option<&PersistedSelections>,
) -> Result<(), String> {
    let trigger_input_name = resolve_port_name(
        config
            .trigger_input_name
            .as_deref()
            .or(config.input_name.as_deref()),
        persisted.and_then(|state| state.trigger_input_name.as_deref()),
        input_names,
        "trigger input",
    )?;
    let output_name = resolve_port_name(
        config.output_name.as_deref(),
        persisted.and_then(|state| state.output_name.as_deref()),
        output_names,
        "output",
    )?;
    let return_input_name = resolve_port_name(
        config.return_input_name.as_deref(),
        persisted.and_then(|state| state.return_input_name.as_deref()),
        input_names,
        "return input",
    )?;

    save_persisted_selections(&PersistedSelections {
        run_mode: Some(PersistedRunMode::DeviceInitiated),
        input_name: None,
        output_name: Some(output_name.clone()),
        trigger_input_name: Some(trigger_input_name.clone()),
        return_input_name: Some(return_input_name.clone()),
    });

    let (trigger_receiver, _trigger_connection) = connect_input_by_name(
        &trigger_input_name,
        "trekr-midi-loopback-trigger-input-connection",
        "trigger",
    )?;
    let (return_receiver, _return_connection) = connect_input_by_name(
        &return_input_name,
        "trekr-midi-loopback-return-input-connection",
        "return",
    )?;
    let mut output_connection = connect_output_by_name(&output_name)?;

    println!(
        "starting device-initiated loopback test output_channel={} warmup={} count={} interval_ms={} timeout_ms={} trigger_input='{}' return_input='{}' output='{}'",
        config.channel,
        config.warmup_count,
        config.count,
        config.interval_ms,
        config.timeout_ms,
        trigger_input_name,
        return_input_name,
        output_name
    );
    println!("  play a note on the trigger device to start each sample");

    for _ in 0..config.warmup_count {
        let _ = send_and_measure_device_initiated(
            config,
            &mut output_connection,
            &trigger_receiver,
            &return_receiver,
            false,
        )?;
    }

    let mut samples = Vec::with_capacity(config.count as usize);
    for _ in 0..config.count {
        let sample = send_and_measure_device_initiated(
            config,
            &mut output_connection,
            &trigger_receiver,
            &return_receiver,
            true,
        )?;
        samples.push(sample);
    }

    print_summary(&samples);
    Ok(())
}

fn print_summary(samples: &[LoopbackSample]) {
    let stats = summarize(samples);
    println!("completed loopback test");
    println!("  samples: {}", stats.count);
    println!("  min_ms: {:.3}", stats.min_ms);
    println!("  p50_ms: {:.3}", stats.p50_ms);
    println!("  p95_ms: {:.3}", stats.p95_ms);
    println!("  p99_ms: {:.3}", stats.p99_ms);
    println!("  max_ms: {:.3}", stats.max_ms);
    println!("  mean_ms: {:.3}", stats.mean_ms);
}

fn resolve_port_name(
    configured_name: Option<&str>,
    default_name: Option<&str>,
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

    if let Some(name) = default_name {
        if available_names.iter().any(|available| available == name)
            && confirm_default(&format!("use last {port_kind} ({name})"), true)?
        {
            return Ok(name.to_string());
        }
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

fn send_and_measure_self_send(
    config: &Config,
    output_connection: &mut MidiOutputConnection,
    input_receiver: &Receiver<MidiObservedEvent>,
    pitch: u8,
    print_sample: bool,
) -> Result<LoopbackSample, String> {
    drain_pending(input_receiver);

    let note_on = MidiNoteOn {
        channel: config.channel,
        pitch,
        velocity: config.velocity,
    };
    let status_on = status_byte(0x90, note_on.channel);
    let status_off = status_byte(0x80, note_on.channel);
    let sent_at = Instant::now();
    println!(
        "  sent note-on pitch={} velocity={} channel={} at {:?}",
        note_on.pitch, note_on.velocity, note_on.channel, sent_at
    );
    output_connection
        .send(&[status_on, note_on.pitch, note_on.velocity])
        .map_err(|error| {
            format!(
                "failed sending note-on for pitch {}: {error}",
                note_on.pitch
            )
        })?;

    let timeout_ms = Some(config.timeout_ms.max(1));
    let received =
        wait_for_matching_note_on(input_receiver, note_on, timeout_ms, sent_at, "return")?;
    let sample = LoopbackSample {
        pitch,
        latency: received.received_at.saturating_duration_since(sent_at),
    };

    thread::sleep(Duration::from_millis(config.note_length_ms.max(1)));
    println!(
        "  sent note-off pitch={} channel={}",
        note_on.pitch, note_on.channel
    );
    output_connection
        .send(&[status_off, note_on.pitch, 0])
        .map_err(|error| {
            format!(
                "failed sending note-off for pitch {}: {error}",
                note_on.pitch
            )
        })?;

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

fn send_and_measure_device_initiated(
    config: &Config,
    output_connection: &mut MidiOutputConnection,
    trigger_receiver: &Receiver<MidiObservedEvent>,
    return_receiver: &Receiver<MidiObservedEvent>,
    print_sample: bool,
) -> Result<LoopbackSample, String> {
    drain_pending(trigger_receiver);
    drain_pending(return_receiver);

    let trigger =
        wait_for_any_note_on(trigger_receiver, Some(config.timeout_ms.max(1)), "trigger")?;
    let trigger_note = trigger
        .note_on
        .ok_or_else(|| "trigger wait returned non-note-on event unexpectedly".to_string())?;

    let forwarded_note = MidiNoteOn {
        channel: config.channel,
        pitch: trigger_note.pitch,
        velocity: trigger_note.velocity,
    };
    let forwarded_at = Instant::now();
    println!(
        "  forwarding trigger pitch={} velocity={} from channel {} to output channel {} at {:?}",
        trigger_note.pitch,
        trigger_note.velocity,
        trigger_note.channel,
        forwarded_note.channel,
        forwarded_at
    );
    output_connection
        .send(&[
            status_byte(0x90, forwarded_note.channel),
            forwarded_note.pitch,
            forwarded_note.velocity,
        ])
        .map_err(|error| {
            format!(
                "failed forwarding note-on for pitch {}: {error}",
                forwarded_note.pitch
            )
        })?;

    let received = wait_for_matching_note_on(
        return_receiver,
        forwarded_note,
        Some(config.timeout_ms.max(1)),
        forwarded_at,
        "return",
    )?;
    let sample = LoopbackSample {
        pitch: forwarded_note.pitch,
        latency: received
            .received_at
            .saturating_duration_since(trigger.received_at),
    };

    thread::sleep(Duration::from_millis(config.note_length_ms.max(1)));
    println!(
        "  sent forwarded note-off pitch={} channel={}",
        forwarded_note.pitch, forwarded_note.channel
    );
    output_connection
        .send(&[
            status_byte(0x80, forwarded_note.channel),
            forwarded_note.pitch,
            0,
        ])
        .map_err(|error| {
            format!(
                "failed forwarding note-off for pitch {}: {error}",
                forwarded_note.pitch
            )
        })?;

    if print_sample {
        println!(
            "  sample pitch={} trigger_to_return_ms={:.3}",
            sample.pitch,
            duration_ms(sample.latency)
        );
    }

    thread::sleep(Duration::from_millis(config.interval_ms));
    Ok(sample)
}

fn wait_for_any_note_on(
    input_receiver: &Receiver<MidiObservedEvent>,
    timeout_ms: Option<u64>,
    expected_source: &str,
) -> Result<MidiObservedEvent, String> {
    let deadline = timeout_ms.map(|timeout| Instant::now() + Duration::from_millis(timeout));
    loop {
        let event = receive_event(input_receiver, deadline, expected_source)?;
        log_observed_event(&event, Instant::now(), expected_source);
        if event.note_on.is_some() {
            return Ok(event);
        }
    }
}

fn wait_for_matching_note_on(
    input_receiver: &Receiver<MidiObservedEvent>,
    expected_note: MidiNoteOn,
    timeout_ms: Option<u64>,
    sent_at: Instant,
    expected_source: &str,
) -> Result<MidiObservedEvent, String> {
    let deadline = timeout_ms.map(|timeout| Instant::now() + Duration::from_millis(timeout));
    loop {
        let event = receive_event(input_receiver, deadline, expected_source)?;
        log_observed_event(&event, sent_at, expected_source);
        if let Some(note_on) = event.note_on {
            if note_on.pitch == expected_note.pitch && note_on.velocity == expected_note.velocity {
                println!(
                    "  MATCH source={} pitch={} velocity={} latency_ms={:.3}",
                    expected_source,
                    note_on.pitch,
                    note_on.velocity,
                    duration_ms(event.received_at.saturating_duration_since(sent_at))
                );
                return Ok(event);
            }
        }
    }
}

fn receive_event(
    input_receiver: &Receiver<MidiObservedEvent>,
    deadline: Option<Instant>,
    expected_source: &str,
) -> Result<MidiObservedEvent, String> {
    match deadline {
        Some(deadline) => loop {
            let now = Instant::now();
            if now >= deadline {
                return Err(format!(
                    "timed out waiting for {expected_source} MIDI activity"
                ));
            }
            let remaining = deadline.saturating_duration_since(now);
            match input_receiver.recv_timeout(remaining) {
                Ok(event) => return Ok(event),
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(format!("{expected_source} MIDI input disconnected"));
                }
            }
        },
        None => input_receiver
            .recv()
            .map_err(|_| format!("{expected_source} MIDI input disconnected")),
    }
}

fn log_observed_event(event: &MidiObservedEvent, sent_at: Instant, expected_source: &str) {
    println!(
        "  {} rx source={} bytes={:?} note_on={} channel={:?} pitch={:?} velocity={:?} age_ms={:.3}",
        expected_source,
        event.source_label,
        event.message_bytes,
        event.note_on.is_some(),
        event.note_on.map(|note| note.channel),
        event.note_on.map(|note| note.pitch),
        event.note_on.map(|note| note.velocity),
        duration_ms(event.received_at.saturating_duration_since(sent_at))
    );
}

fn drain_pending(input_receiver: &Receiver<MidiObservedEvent>) {
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

fn connect_input_by_name(
    target_name: &str,
    connection_name: &str,
    source_label: &str,
) -> Result<(Receiver<MidiObservedEvent>, MidiInputConnection<()>), String> {
    let mut midi_in = MidiInput::new(connection_name).map_err(|error| error.to_string())?;
    midi_in.ignore(Ignore::None);
    let (sender, receiver) = mpsc::channel();
    let port = midi_in
        .ports()
        .into_iter()
        .find(|port| midi_in.port_name(port).ok().as_deref() == Some(target_name))
        .ok_or_else(|| format!("MIDI input port '{}' not found", target_name))?;
    let source_label = source_label.to_string();

    let connection = midi_in
        .connect(
            &port,
            connection_name,
            move |_timestamp, message, _state| {
                let received_at = Instant::now();
                println!("  {} midi input activity bytes={:?}", source_label, message);
                let _ = sender.send(MidiObservedEvent {
                    source_label: source_label.clone(),
                    note_on: parse_note_on(message),
                    message_bytes: message.to_vec(),
                    received_at,
                });
            },
            (),
        )
        .map_err(|error| error.to_string())?;

    Ok((receiver, connection))
}

fn connect_output_by_name(target_name: &str) -> Result<MidiOutputConnection, String> {
    let midi_out = MidiOutput::new("trekr-midi-loopback-output-connection")
        .map_err(|error| error.to_string())?;
    let port = midi_out
        .ports()
        .into_iter()
        .find(|port| midi_out.port_name(port).ok().as_deref() == Some(target_name))
        .ok_or_else(|| format!("MIDI output port '{}' not found", target_name))?;

    midi_out
        .connect(&port, "trekr-midi-loopback-output-connection")
        .map_err(|error| error.to_string())
}

fn parse_note_on(message: &[u8]) -> Option<MidiNoteOn> {
    let status = *message.first()?;
    if status & 0xF0 != 0x90 {
        return None;
    }
    let pitch = *message.get(1)?;
    let velocity = *message.get(2)?;
    (velocity > 0).then_some(MidiNoteOn {
        channel: (status & 0x0F) + 1,
        pitch,
        velocity,
    })
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
        trigger_input_name: None,
        return_input_name: None,
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
            "--trigger-input-name" => {
                config.trigger_input_name = Some(
                    args.next()
                        .ok_or_else(|| "missing value for --trigger-input-name".to_string())?,
                );
            }
            "--return-input-name" => {
                config.return_input_name = Some(
                    args.next()
                        .ok_or_else(|| "missing value for --return-input-name".to_string())?,
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
        "  cargo run --bin trekr-midi-loopback-latency -- --trigger-input-name <name> --output-name <name> --return-input-name <name> [options]"
    );
    println!(
        "  cargo run --bin trekr-midi-loopback-latency -- [options]    # prompts interactively for mode and ports when omitted"
    );
    println!("Options:");
    println!("  --list-only                 Enumerate native MIDI ports and exit");
    println!("  --input-name <name>         Self-send loopback input by exact port name");
    println!("  --output-name <name>        MIDI output by exact port name");
    println!("  --trigger-input-name <name> Device-initiated trigger input by exact port name");
    println!("  --return-input-name <name>  Device-initiated return input by exact port name");
    println!("  --note <0-127>              Base self-send test note. Default: 60");
    println!("  --velocity <0-127>          Default self-send velocity: 100");
    println!("  --channel <1-16>            Output channel. Default: 1");
    println!("  --count <n>                 Measured samples. Default: 32");
    println!("  --warmup-count <n>          Warmup samples not included in stats. Default: 4");
    println!("  --interval-ms <ms>          Delay between measured pings. Default: 40");
    println!("  --note-length-ms <ms>       Note hold before note-off. Default: 10");
    println!("  --timeout-ms <ms>           Self-send and trigger wait timeout. Default: 500");
}

fn persisted_selections_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".trekr-midi-loopback-latency.json")
}

fn load_persisted_selections() -> Option<PersistedSelections> {
    let path = persisted_selections_path();
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn save_persisted_selections(state: &PersistedSelections) {
    let path = persisted_selections_path();
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = fs::write(path, json);
    }
}

fn confirm_default(prompt: &str, default_yes: bool) -> Result<bool, String> {
    let suffix = if default_yes { "[Y/n]" } else { "[y/N]" };
    loop {
        print!("{prompt} {suffix}: ");
        io::stdout()
            .flush()
            .map_err(|error| format!("failed flushing confirmation prompt: {error}"))?;
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .map_err(|error| format!("failed reading confirmation prompt: {error}"))?;
        match line.trim().to_ascii_lowercase().as_str() {
            "" => return Ok(default_yes),
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            other => eprintln!("invalid selection '{other}': enter y or n"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LoopbackSample, MidiNoteOn, PersistedSelections, RunMode, detect_run_mode, duration_ms,
        parse_note_on, percentile, prompt_for_port_selection, resolve_port_name, summarize,
    };
    use std::time::Duration;

    #[test]
    fn detect_run_mode_switches_when_trigger_path_is_configured() {
        let mut config = super::Config {
            list_only: false,
            input_name: None,
            output_name: None,
            trigger_input_name: Some("Trigger".to_string()),
            return_input_name: None,
            note: 60,
            velocity: 100,
            channel: 1,
            count: 1,
            warmup_count: 0,
            interval_ms: 0,
            note_length_ms: 0,
            timeout_ms: 100,
        };
        assert_eq!(
            detect_run_mode(&config, None).unwrap(),
            RunMode::DeviceInitiated
        );
        config.trigger_input_name = None;
        config.input_name = Some("Input".to_string());
        assert_eq!(detect_run_mode(&config, None).unwrap(), RunMode::SelfSend);
    }

    #[test]
    fn detect_run_mode_uses_persisted_default_when_no_mode_args_are_set() {
        let config = super::Config {
            list_only: false,
            input_name: None,
            output_name: None,
            trigger_input_name: None,
            return_input_name: None,
            note: 60,
            velocity: 100,
            channel: 1,
            count: 1,
            warmup_count: 0,
            interval_ms: 0,
            note_length_ms: 0,
            timeout_ms: 100,
        };
        let persisted = PersistedSelections {
            run_mode: Some(super::PersistedRunMode::DeviceInitiated),
            ..PersistedSelections::default()
        };
        assert_eq!(
            detect_run_mode(&config, Some(&persisted)).unwrap(),
            RunMode::DeviceInitiated
        );
    }

    #[test]
    fn parse_note_on_extracts_channel_pitch_and_velocity() {
        assert_eq!(
            parse_note_on(&[0x92, 64, 100]),
            Some(MidiNoteOn {
                channel: 3,
                pitch: 64,
                velocity: 100,
            })
        );
        assert_eq!(parse_note_on(&[0x82, 64, 0]), None);
    }

    #[test]
    fn resolve_port_name_accepts_existing_configured_name() {
        let ports = vec!["In A".to_string(), "In B".to_string()];
        assert_eq!(
            resolve_port_name(Some("In B"), None, &ports, "input").unwrap(),
            "In B".to_string()
        );
    }

    #[test]
    fn resolve_port_name_rejects_missing_configured_name() {
        let ports = vec!["In A".to_string()];
        assert!(resolve_port_name(Some("Missing"), None, &ports, "input").is_err());
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
