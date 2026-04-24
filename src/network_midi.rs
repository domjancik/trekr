use crate::midi_io::{
    MidiInputEvent, MidiInputMessage, MidiPortRef, MidiTransportProtocol, NetworkMidiEndpoint,
};
use flume::{Receiver as FlumeReceiver, RecvTimeoutError};
use mdns_sd::{ResolvedService, ServiceDaemon, ServiceEvent};
use std::collections::HashMap;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const APPLE_MIDI_SERVICE_TYPE: &str = "_apple-midi._udp.local.";

pub struct NetworkMidiRuntime {
    mdns: Option<ServiceDaemon>,
    browse_receiver: Option<FlumeReceiver<ServiceEvent>>,
    discovered_ports: HashMap<String, MidiPortRef>,
    input_sender: Sender<MidiInputEvent>,
    input_receiver: Receiver<MidiInputEvent>,
    connections: HashMap<String, NetworkMidiConnection>,
    #[cfg(test)]
    requested_ports: Vec<String>,
    #[cfg(test)]
    sent_messages: Vec<(String, u8, u8, Option<u8>)>,
}

#[derive(Debug)]
struct NetworkMidiConnection {
    data_socket: UdpSocket,
    remote_data_addr: SocketAddr,
    ssrc: u32,
    sequence: u16,
    shutdown_sender: Sender<()>,
}

impl Default for NetworkMidiRuntime {
    fn default() -> Self {
        let (input_sender, input_receiver) = mpsc::channel();
        let (mdns, browse_receiver) = match ServiceDaemon::new() {
            Ok(mdns) => {
                let browse_receiver = mdns.browse(APPLE_MIDI_SERVICE_TYPE).ok();
                (Some(mdns), browse_receiver)
            }
            Err(_) => (None, None),
        };

        Self {
            mdns,
            browse_receiver,
            discovered_ports: HashMap::new(),
            input_sender,
            input_receiver,
            connections: HashMap::new(),
            #[cfg(test)]
            requested_ports: Vec::new(),
            #[cfg(test)]
            sent_messages: Vec::new(),
        }
    }
}

impl Drop for NetworkMidiRuntime {
    fn drop(&mut self) {
        self.connections.clear();
        if let Some(mdns) = self.mdns.take() {
            let _ = mdns.shutdown();
        }
    }
}

impl NetworkMidiRuntime {
    pub fn poll_discovery(&mut self) -> bool {
        let mut changed = false;
        let Some(receiver) = &self.browse_receiver else {
            return false;
        };

        loop {
            match receiver.recv_timeout(Duration::from_millis(0)) {
                Ok(ServiceEvent::ServiceResolved(info)) => {
                    let info = *info;
                    let key = info.get_fullname().to_string();
                    let port = port_from_resolved_service(&info);
                    let replace = self
                        .discovered_ports
                        .get(&key)
                        .is_none_or(|existing| existing != &port);
                    if replace {
                        self.discovered_ports.insert(key, port);
                        changed = true;
                    }
                }
                Ok(ServiceEvent::ServiceRemoved(_, fullname)) => {
                    if self.discovered_ports.remove(&fullname).is_some() {
                        self.connections.remove(&fullname);
                        changed = true;
                    }
                }
                Ok(_) => {}
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => {
                    self.browse_receiver = None;
                    break;
                }
            }
        }

        changed
    }

    pub fn available_ports(&mut self) -> Vec<MidiPortRef> {
        self.poll_discovery();
        let mut ports = self.discovered_ports.values().cloned().collect::<Vec<_>>();
        ports.sort_by(|left, right| left.name.cmp(&right.name));
        ports
    }

    pub fn sync_ports(&mut self, ports: &[MidiPortRef]) {
        self.poll_discovery();
        let wanted = ports
            .iter()
            .filter(|port| port.protocol == MidiTransportProtocol::RtpMidiNative)
            .cloned()
            .collect::<Vec<_>>();
        #[cfg(test)]
        {
            self.requested_ports = wanted.iter().map(|port| port.name.clone()).collect();
            self.requested_ports.sort();
        }

        self.connections.retain(|key, connection| {
            let keep = wanted
                .iter()
                .any(|port| port.network_key().as_deref() == Some(key.as_str()));
            if !keep {
                connection.shutdown();
            }
            keep
        });

        for port in wanted {
            let _ = self.ensure_connection(&port);
        }
    }

    pub fn drain_events(&mut self) -> Vec<MidiInputEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.input_receiver.try_recv() {
            events.push(event);
        }
        events
    }

    pub fn send_note_on(
        &mut self,
        port: &MidiPortRef,
        channel: u8,
        pitch: u8,
        velocity: u8,
    ) -> Result<(), String> {
        #[cfg(test)]
        self.sent_messages
            .push((port.name.clone(), channel, pitch, Some(velocity)));
        let connection = self.ensure_connection(port)?;
        connection.send_short_message(short_message(0x90, channel.clamp(1, 16), pitch, velocity))
    }

    pub fn send_note_off(
        &mut self,
        port: &MidiPortRef,
        channel: u8,
        pitch: u8,
    ) -> Result<(), String> {
        #[cfg(test)]
        self.sent_messages
            .push((port.name.clone(), channel, pitch, None));
        let connection = self.ensure_connection(port)?;
        connection.send_short_message(short_message(0x80, channel.clamp(1, 16), pitch, 0))
    }

    pub fn send_all_notes_off(&mut self, port: &MidiPortRef, channel: u8) -> Result<(), String> {
        #[cfg(test)]
        self.sent_messages
            .push((port.name.clone(), channel, 123, None));
        let connection = self.ensure_connection(port)?;
        connection.send_short_message(short_message(0xB0, channel.clamp(1, 16), 123, 0))
    }

    #[cfg(test)]
    pub fn requested_port_names(&self) -> Vec<String> {
        self.requested_ports.clone()
    }

    #[cfg(test)]
    pub fn sent_messages(&self) -> Vec<(String, u8, u8, Option<u8>)> {
        self.sent_messages.clone()
    }

    fn ensure_connection(
        &mut self,
        port: &MidiPortRef,
    ) -> Result<&mut NetworkMidiConnection, String> {
        let key = port.network_key().unwrap_or_else(|| port.name.clone());
        if !self.connections.contains_key(&key) {
            let connection = NetworkMidiConnection::connect(port, self.input_sender.clone())?;
            self.connections.insert(key.clone(), connection);
        }

        self.connections
            .get_mut(&key)
            .ok_or_else(|| format!("missing RTP-MIDI connection for {}", port.name))
    }
}

impl NetworkMidiConnection {
    fn connect(port: &MidiPortRef, input_sender: Sender<MidiInputEvent>) -> Result<Self, String> {
        let endpoint = port
            .network_endpoint
            .as_ref()
            .ok_or_else(|| format!("missing RTP endpoint metadata for {}", port.name))?;

        let control_addr = resolve_addr(&endpoint.host, endpoint.control_port)?;
        let data_addr = resolve_addr(&endpoint.host, endpoint.data_port)?;
        let control_socket = bind_ephemeral_socket()?;
        let data_socket = bind_ephemeral_socket()?;
        let initiator_token = random_u32();
        let ssrc = random_u32();

        send_invite(
            &control_socket,
            control_addr,
            initiator_token,
            ssrc,
            "trekr",
        )?;
        let _ = recv_control_response(&control_socket);
        send_invite(&data_socket, data_addr, initiator_token, ssrc, "trekr")?;
        let _ = recv_control_response(&data_socket);

        let (shutdown_sender, shutdown_receiver) = mpsc::channel();
        let recv_socket = data_socket
            .try_clone()
            .map_err(|error| format!("failed to clone RTP-MIDI socket: {error}"))?;
        let recv_port = port.clone();
        thread::Builder::new()
            .name(format!(
                "trekr-rtpmidi-{}",
                sanitize_thread_name(&port.name)
            ))
            .spawn(move || {
                receive_rtp_midi_loop(recv_socket, recv_port, input_sender, shutdown_receiver);
            })
            .map_err(|error| error.to_string())?;

        Ok(Self {
            data_socket,
            remote_data_addr: data_addr,
            ssrc,
            sequence: 0,
            shutdown_sender,
        })
    }

    fn send_short_message(&mut self, message: [u8; 3]) -> Result<(), String> {
        self.sequence = self.sequence.wrapping_add(1);
        let packet = build_rtp_midi_packet(self.sequence, self.ssrc, &message);
        self.data_socket
            .send_to(&packet, self.remote_data_addr)
            .map_err(|error| format!("failed to send RTP-MIDI packet: {error}"))?;
        Ok(())
    }

    fn shutdown(&self) {
        let _ = self.shutdown_sender.send(());
    }
}

fn port_from_resolved_service(info: &ResolvedService) -> MidiPortRef {
    let instance_name = info
        .get_fullname()
        .trim_end_matches("._apple-midi._udp.local.")
        .to_string();
    let host = info
        .get_addresses_v4()
        .into_iter()
        .next()
        .map(IpAddr::V4)
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));
    let control_port = info.get_port();
    let data_port = control_port.saturating_add(1);

    MidiPortRef::rtp_midi(
        &format!("{instance_name} (RTP)"),
        NetworkMidiEndpoint {
            key: info.get_fullname().to_string(),
            host: host.to_string(),
            control_port,
            data_port,
            service_name: Some(info.get_fullname().to_string()),
            host_name: Some(info.get_hostname().to_string()),
        },
    )
}

fn bind_ephemeral_socket() -> Result<UdpSocket, String> {
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|error| error.to_string())?;
    socket
        .set_read_timeout(Some(Duration::from_millis(250)))
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

fn send_invite(
    socket: &UdpSocket,
    addr: SocketAddr,
    initiator_token: u32,
    ssrc: u32,
    session_name: &str,
) -> Result<(), String> {
    let packet = build_control_packet(b"IN", initiator_token, ssrc, session_name);
    socket
        .send_to(&packet, addr)
        .map_err(|error| format!("failed to send RTP-MIDI invitation: {error}"))?;
    Ok(())
}

fn recv_control_response(socket: &UdpSocket) -> Result<(), String> {
    let mut buffer = [0u8; 1500];
    match socket.recv_from(&mut buffer) {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => Ok(()),
        Err(error) => Err(format!("failed to receive RTP-MIDI response: {error}")),
    }
}

fn receive_rtp_midi_loop(
    socket: UdpSocket,
    port: MidiPortRef,
    sender: Sender<MidiInputEvent>,
    shutdown_receiver: Receiver<()>,
) {
    let _ = socket.set_read_timeout(Some(Duration::from_millis(250)));
    let mut buffer = [0u8; 1500];
    loop {
        if shutdown_receiver.try_recv().is_ok() {
            break;
        }
        match socket.recv_from(&mut buffer) {
            Ok((len, _)) => {
                for event in parse_rtp_midi_input_events(&port, &buffer[..len]) {
                    let _ = sender.send(event);
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) => {}
            Err(_) => break,
        }
    }
}

fn parse_rtp_midi_input_events(port: &MidiPortRef, packet: &[u8]) -> Vec<MidiInputEvent> {
    if packet.len() <= 12 {
        return Vec::new();
    }

    let Some((command_length, command_offset)) = parse_rtp_midi_command_length(&packet[12..])
    else {
        return Vec::new();
    };
    let command_start = 12 + command_offset;
    let command_end = command_start
        .saturating_add(command_length)
        .min(packet.len());
    if command_start >= command_end {
        return Vec::new();
    }

    parse_midi_command_stream(port, &packet[command_start..command_end])
}

fn parse_rtp_midi_command_length(payload: &[u8]) -> Option<(usize, usize)> {
    let mut value = 0usize;
    let mut consumed = 0usize;
    for byte in payload {
        consumed += 1;
        value = (value << 7) | usize::from(byte & 0x7F);
        if byte & 0x80 == 0 {
            return Some((value, consumed));
        }
    }
    None
}

fn parse_midi_command_stream(port: &MidiPortRef, bytes: &[u8]) -> Vec<MidiInputEvent> {
    let mut events = Vec::new();
    let mut cursor = 0usize;
    while cursor + 2 < bytes.len() {
        let status = bytes[cursor];
        if status < 0x80 {
            break;
        }
        let data1 = bytes[cursor + 1];
        let data2 = bytes[cursor + 2];
        cursor += 3;
        let channel = (status & 0x0F) + 1;
        let message = match status & 0xF0 {
            0x80 => MidiInputMessage::NoteOff { pitch: data1 },
            0x90 if data2 == 0 => MidiInputMessage::NoteOff { pitch: data1 },
            0x90 => MidiInputMessage::NoteOn {
                pitch: data1,
                velocity: data2,
            },
            0xB0 => MidiInputMessage::ControlChange {
                controller: data1,
                value: data2,
            },
            _ => continue,
        };
        events.push(MidiInputEvent {
            port: port.clone(),
            channel,
            message,
        });
    }
    events
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

fn build_rtp_midi_packet(sequence: u16, ssrc: u32, message: &[u8; 3]) -> Vec<u8> {
    let timestamp = random_u32();
    let mut packet = Vec::with_capacity(16);
    packet.extend_from_slice(&[0x80, 0x61]);
    packet.extend_from_slice(&sequence.to_be_bytes());
    packet.extend_from_slice(&timestamp.to_be_bytes());
    packet.extend_from_slice(&ssrc.to_be_bytes());
    packet.push(3);
    packet.extend_from_slice(message);
    packet
}

fn short_message(base_status: u8, channel: u8, data1: u8, data2: u8) -> [u8; 3] {
    [
        base_status | channel.saturating_sub(1).min(15),
        data1,
        data2,
    ]
}

fn random_u32() -> u32 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0));
    (duration.subsec_nanos() ^ (duration.as_secs() as u32)).wrapping_mul(1664525)
}

fn sanitize_thread_name(name: &str) -> String {
    name.chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() {
                char
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rtp_midi_packet_extracts_note_and_cc_messages() {
        let port = MidiPortRef::rtp_midi(
            "Peer (RTP)",
            NetworkMidiEndpoint {
                key: "peer".to_string(),
                host: "127.0.0.1".to_string(),
                control_port: 5004,
                data_port: 5005,
                service_name: None,
                host_name: None,
            },
        );
        let packet = build_rtp_midi_packet(1, 42, &[0x90, 60, 100]);
        let cc_packet = build_rtp_midi_packet(2, 42, &[0xB0, 74, 127]);

        let note_events = parse_rtp_midi_input_events(&port, &packet);
        let cc_events = parse_rtp_midi_input_events(&port, &cc_packet);

        assert_eq!(note_events.len(), 1);
        assert_eq!(
            note_events[0].message,
            MidiInputMessage::NoteOn {
                pitch: 60,
                velocity: 100
            }
        );
        assert_eq!(cc_events.len(), 1);
        assert_eq!(
            cc_events[0].message,
            MidiInputMessage::ControlChange {
                controller: 74,
                value: 127
            }
        );
    }
}
