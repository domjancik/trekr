use std::env;
use std::process::Command;

fn optional_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[test]
#[ignore = "requires live network/firewall access and may fail due to host policy rather than code defects"]
fn mdns_probe_starts_in_live_environment() {
    let binary = env!("CARGO_BIN_EXE_trekr-mdns-probe");
    let output = Command::new(binary)
        .args([
            "--browse-only",
            "--service-type",
            "_apple-midi._udp",
            "--duration-secs",
            "2",
        ])
        .output()
        .expect("mDNS probe binary should launch");

    assert!(
        output.status.success(),
        "live mDNS probe failed; this may indicate Windows firewall, multicast policy, or missing responder availability instead of an RTP-MIDI code defect.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "requires configured live RTP-MIDI peer, UDP reachability, and firewall access"]
fn rtpmidi_smoke_reaches_configured_peer() {
    let Some(host) = optional_env("TREKR_RTPMIDI_TEST_HOST") else {
        eprintln!("skipping live RTP-MIDI smoke test because TREKR_RTPMIDI_TEST_HOST was not set");
        return;
    };
    let control_port =
        optional_env("TREKR_RTPMIDI_TEST_CONTROL_PORT").unwrap_or_else(|| "5004".to_string());
    let note = optional_env("TREKR_RTPMIDI_TEST_NOTE").unwrap_or_else(|| "60".to_string());

    let binary = env!("CARGO_BIN_EXE_trekr-rtpmidi-smoke");
    let output = Command::new(binary)
        .args([
            "--host",
            &host,
            "--control-port",
            &control_port,
            "--note",
            &note,
            "--velocity",
            "100",
            "--hold-ms",
            "100",
        ])
        .output()
        .expect("RTP-MIDI smoke binary should launch");

    assert!(
        output.status.success(),
        "live RTP-MIDI smoke test failed; this may indicate firewall policy, unreachable peer, peer-side session rejection, or other environment constraints rather than a trekr regression.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
