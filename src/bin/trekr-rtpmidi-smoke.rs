use std::io;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct Config {
    host: String,
    control_port: u16,
    data_port: Option<u16>,
    session_name: String,
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
        eprintln!("RTP-MIDI smoke failed: {error}");
        std::process::exit(1);
    }
}

fn run(config: &Config) -> Result<(), String> {
    let control_addr = resolve_addr(&config.host, config.control_port)?;
    let data_port = config
        .data_port
        .unwrap_or_else(|| config.control_port.saturating_add(1));
    let data_addr = resolve_addr(&config.host, data_port)?;

    println!("rtpmidi smoke sender");
    println!("  control: {control_addr}");
    println!("  data:    {data_addr}");

    let control_socket = bind_ephemeral_socket()?;
    let data_socket = bind_ephemeral_socket()?;

    let initiator_token = random_u32();
    let ssrc = random_u32();

    send_invite(
        &control_socket,
        control_addr,
        initiator_token,
        ssrc,
        &config.session_name,
        "control",
    )?;
    let _ = recv_control_response(&control_socket, "control");

    send_invite(
        &data_socket,
        data_addr,
        initiator_token,
        ssrc,
        &config.session_name,
        "data",
    )?;
    let _ = recv_control_response(&data_socket, "data");

    let mut sequence = 1u16;
    send_rtp_midi_note(
        &data_socket,
        data_addr,
        sequence,
        ssrc,
        true,
        config.channel,
        config.note,
        config.velocity,
    )?;
    println!(
        "  sent note-on ch{} note={} vel={}",
        config.channel, config.note, config.velocity
    );

    std::thread::sleep(Duration::from_millis(config.hold_ms.max(1)));

    sequence = sequence.wrapping_add(1);
    send_rtp_midi_note(
        &data_socket,
        data_addr,
        sequence,
        ssrc,
        false,
        config.channel,
        config.note,
        0,
    )?;
    println!("  sent note-off ch{} note={}", config.channel, config.note);

    println!("rtpmidi smoke sender finished");
    Ok(())
}

fn send_invite(
    socket: &UdpSocket,
    addr: SocketAddr,
    initiator_token: u32,
    ssrc: u32,
    session_name: &str,
    label: &str,
) -> Result<(), String> {
    let packet = build_control_packet(b"IN", initiator_token, ssrc, session_name);
    socket
        .send_to(&packet, addr)
        .map_err(|error| format!("failed to send {label} invitation: {error}"))?;
    println!("  sent {label} invitation ({})", packet.len());
    Ok(())
}

fn recv_control_response(socket: &UdpSocket, label: &str) -> Result<(), String> {
    let mut buffer = [0u8; 1500];
    match socket.recv_from(&mut buffer) {
        Ok((len, from)) => {
            let command = if len >= 4 {
                String::from_utf8_lossy(&buffer[2..4]).to_string()
            } else {
                "??".to_string()
            };
            println!("  {label} response from {from}: cmd={command} bytes={len}");
            Ok(())
        }
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
            println!("  no {label} response received before timeout");
            Ok(())
        }
        Err(error) => Err(format!("failed receiving {label} response: {error}")),
    }
}

fn send_rtp_midi_note(
    socket: &UdpSocket,
    addr: SocketAddr,
    sequence: u16,
    ssrc: u32,
    note_on: bool,
    channel: u8,
    note: u8,
    velocity: u8,
) -> Result<(), String> {
    let status = if note_on { 0x90 } else { 0x80 } | channel.saturating_sub(1).min(15);
    let timestamp = random_u32();

    let mut packet = Vec::with_capacity(16);
    packet.extend_from_slice(&[0x80, 0x61]);
    packet.extend_from_slice(&sequence.to_be_bytes());
    packet.extend_from_slice(&timestamp.to_be_bytes());
    packet.extend_from_slice(&ssrc.to_be_bytes());
    packet.push(0x03);
    packet.push(status);
    packet.push(note);
    packet.push(velocity);

    socket
        .send_to(&packet, addr)
        .map_err(|error| format!("failed to send RTP-MIDI note packet: {error}"))?;
    Ok(())
}

fn build_control_packet(command: &[u8; 2], initiator_token: u32, ssrc: u32, name: &str) -> Vec<u8> {
    let mut packet = Vec::with_capacity(16 + name.len());
    packet.extend_from_slice(&[0xFF, 0xFF]);
    packet.extend_from_slice(command);
    packet.extend_from_slice(&2u32.to_be_bytes());
    packet.extend_from_slice(&initiator_token.to_be_bytes());
    packet.extend_from_slice(&ssrc.to_be_bytes());
    packet.extend_from_slice(name.as_bytes());
    packet
}

fn bind_ephemeral_socket() -> Result<UdpSocket, String> {
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|error| error.to_string())?;
    socket
        .set_read_timeout(Some(Duration::from_millis(1500)))
        .map_err(|error| error.to_string())?;
    Ok(socket)
}

fn resolve_addr(host: &str, port: u16) -> Result<SocketAddr, String> {
    let mut addrs = (host, port)
        .to_socket_addrs()
        .map_err(|error| format!("failed to resolve {host}:{port}: {error}"))?;
    addrs
        .next()
        .ok_or_else(|| format!("no socket addresses resolved for {host}:{port}"))
}

fn random_u32() -> u32 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0));
    (duration.subsec_nanos() ^ (duration.as_secs() as u32)).wrapping_mul(1664525)
}

fn parse_args() -> Result<Config, String> {
    let mut host = String::new();
    let mut control_port = 5004u16;
    let mut data_port = None;
    let mut session_name = "trekr-rtpmidi-smoke".to_string();
    let mut note = 60u8;
    let mut velocity = 100u8;
    let mut channel = 1u8;
    let mut hold_ms = 250u64;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--host" => {
                host = args
                    .next()
                    .ok_or_else(|| "missing value for --host".to_string())?
            }
            "--control-port" => {
                let raw = args
                    .next()
                    .ok_or_else(|| "missing value for --control-port".to_string())?;
                control_port = raw
                    .parse::<u16>()
                    .map_err(|_| format!("invalid control port '{raw}'"))?;
            }
            "--data-port" => {
                let raw = args
                    .next()
                    .ok_or_else(|| "missing value for --data-port".to_string())?;
                data_port = Some(
                    raw.parse::<u16>()
                        .map_err(|_| format!("invalid data port '{raw}'"))?,
                );
            }
            "--name" => {
                session_name = args
                    .next()
                    .ok_or_else(|| "missing value for --name".to_string())?;
            }
            "--note" => {
                let raw = args
                    .next()
                    .ok_or_else(|| "missing value for --note".to_string())?;
                note = raw
                    .parse::<u8>()
                    .map_err(|_| format!("invalid note '{raw}'"))?;
            }
            "--velocity" => {
                let raw = args
                    .next()
                    .ok_or_else(|| "missing value for --velocity".to_string())?;
                velocity = raw
                    .parse::<u8>()
                    .map_err(|_| format!("invalid velocity '{raw}'"))?;
            }
            "--channel" => {
                let raw = args
                    .next()
                    .ok_or_else(|| "missing value for --channel".to_string())?;
                channel = raw
                    .parse::<u8>()
                    .map_err(|_| format!("invalid channel '{raw}'"))?;
            }
            "--hold-ms" => {
                let raw = args
                    .next()
                    .ok_or_else(|| "missing value for --hold-ms".to_string())?;
                hold_ms = raw
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

    if host.is_empty() {
        return Err("--host is required".to_string());
    }

    Ok(Config {
        host,
        control_port,
        data_port,
        session_name,
        note,
        velocity,
        channel,
        hold_ms,
    })
}

fn print_usage() {
    println!("RTP-MIDI smoke sender");
    println!("Usage: cargo run --bin trekr-rtpmidi-smoke -- --host <ip> [options]");
    println!("Options:");
    println!("  --host <ip/host>            Required target host");
    println!("  --control-port <u16>        Default: 5004");
    println!("  --data-port <u16>           Default: control-port + 1");
    println!("  --name <session-name>       Default: trekr-rtpmidi-smoke");
    println!("  --note <0-127>              Default: 60");
    println!("  --velocity <0-127>          Default: 100");
    println!("  --channel <1-16>            Default: 1");
    println!("  --hold-ms <ms>              Default: 250");
}
