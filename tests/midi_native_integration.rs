use std::env;
use std::process::Command;

fn optional_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[test]
#[ignore = "requires live native MIDI backend access and should be run explicitly, preferably with -- --ignored --test-threads=1"]
fn midi_native_enumeration_smoke_runs_in_live_environment() {
    let binary = env!("CARGO_BIN_EXE_trekr-midi-smoke");
    let output = Command::new(binary)
        .args(["--list-only"])
        .output()
        .expect("native MIDI smoke binary should launch");

    assert!(
        output.status.success(),
        "native MIDI enumeration smoke failed; this may indicate host-specific WinMM/CoreMIDI/ALSA backend issues, missing permissions, or environment instability rather than a trekr regression.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "requires configured live or virtual MIDI port names and should be run explicitly, preferably with -- --ignored --test-threads=1"]
fn midi_native_connection_smoke_reaches_configured_ports() {
    let input_name = optional_env("TREKR_MIDI_TEST_INPUT");
    let output_name = optional_env("TREKR_MIDI_TEST_OUTPUT");

    if input_name.is_none() && output_name.is_none() {
        eprintln!(
            "skipping live native MIDI connection smoke test because TREKR_MIDI_TEST_INPUT and TREKR_MIDI_TEST_OUTPUT were not set"
        );
        return;
    }

    let note = optional_env("TREKR_MIDI_TEST_NOTE").unwrap_or_else(|| "60".to_string());
    let velocity = optional_env("TREKR_MIDI_TEST_VELOCITY").unwrap_or_else(|| "100".to_string());
    let channel = optional_env("TREKR_MIDI_TEST_CHANNEL").unwrap_or_else(|| "1".to_string());
    let hold_ms = optional_env("TREKR_MIDI_TEST_HOLD_MS").unwrap_or_else(|| "100".to_string());

    let binary = env!("CARGO_BIN_EXE_trekr-midi-smoke");
    let mut command = Command::new(binary);
    if let Some(input_name) = input_name.as_deref() {
        command.args(["--input-name", input_name]);
    }
    if let Some(output_name) = output_name.as_deref() {
        command.args(["--output-name", output_name]);
    }
    command.args([
        "--note",
        &note,
        "--velocity",
        &velocity,
        "--channel",
        &channel,
        "--hold-ms",
        &hold_ms,
    ]);

    let output = command
        .output()
        .expect("native MIDI smoke binary should launch");

    assert!(
        output.status.success(),
        "native MIDI connection smoke failed; this may indicate unavailable port names, virtual-device misconfiguration, host MIDI policy, or backend instability rather than a trekr regression.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
