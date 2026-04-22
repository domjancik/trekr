use mdns_sd::{IfKind, ServiceDaemon, ServiceEvent, ServiceInfo};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeMode {
    BrowseOnly,
    AdvertiseOnly,
    Both,
}

#[derive(Debug, Clone)]
struct ProbeConfig {
    service_type: String,
    instance_name: String,
    host_name: String,
    port: u16,
    duration: Option<Duration>,
    mode: ProbeMode,
    verify: bool,
    disable_ipv6: bool,
    include_apple_p2p: bool,
}

fn main() {
    let config = match parse_args() {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}");
            print_usage();
            std::process::exit(2);
        }
    };

    if let Err(error) = run_probe(&config) {
        eprintln!("Probe failed: {error}");
        std::process::exit(1);
    }
}

fn run_probe(config: &ProbeConfig) -> Result<(), String> {
    println!("trekr mDNS probe starting");
    println!("  service type: {}", config.service_type);
    println!("  mode: {:?}", config.mode);
    println!(
        "  duration: {}",
        config
            .duration
            .map(|duration| format!("{}s", duration.as_secs()))
            .unwrap_or_else(|| "until interrupted".to_string())
    );

    let mdns = ServiceDaemon::new().map_err(|error| error.to_string())?;

    if config.disable_ipv6 {
        mdns.disable_interface(IfKind::IPv6)
            .map_err(|error| error.to_string())?;
        println!("  IPv6 interfaces disabled");
    }

    if config.include_apple_p2p {
        mdns.include_apple_p2p(true)
            .map_err(|error| error.to_string())?;
        println!("  Apple P2P interfaces included");
    }

    let mut registered_fullname: Option<String> = None;
    if matches!(config.mode, ProbeMode::AdvertiseOnly | ProbeMode::Both) {
        let properties = [
            ("app", "trekr-mdns-probe"),
            ("probe", "mdns"),
            (
                "mode",
                match config.mode {
                    ProbeMode::BrowseOnly => "browse",
                    ProbeMode::AdvertiseOnly => "advertise",
                    ProbeMode::Both => "both",
                },
            ),
        ];

        let service_info = ServiceInfo::new(
            &config.service_type,
            &config.instance_name,
            &config.host_name,
            "",
            config.port,
            &properties,
        )
        .map_err(|error| error.to_string())?
        .enable_addr_auto();

        let fullname = service_info.get_fullname().to_string();
        mdns.register(service_info)
            .map_err(|error| error.to_string())?;
        println!("  registered: {fullname}");
        registered_fullname = Some(fullname);
    }

    let browse_receiver = if matches!(config.mode, ProbeMode::BrowseOnly | ProbeMode::Both) {
        println!("  browsing started");
        Some(
            mdns.browse(&config.service_type)
                .map_err(|error| error.to_string())?,
        )
    } else {
        None
    };

    let started_at = Instant::now();
    let mut resolved_count = 0usize;

    loop {
        if let Some(limit) = config.duration {
            if started_at.elapsed() >= limit {
                println!("Probe duration reached.");
                break;
            }
        }

        if let Some(receiver) = &browse_receiver {
            match receiver.recv_timeout(Duration::from_millis(500)) {
                Ok(event) => {
                    println!("[{:.3}s] {:?}", started_at.elapsed().as_secs_f32(), event);
                    if let ServiceEvent::ServiceResolved(info) = event {
                        resolved_count += 1;
                        println!(
                            "  resolved host={} port={}",
                            info.get_hostname(),
                            info.get_port()
                        );
                        for address in info.get_addresses_v4() {
                            println!("  address={address}");
                        }
                        for property in info.get_properties() {
                            println!("  txt={property}");
                        }

                        if config.verify && !info.get_addresses_v4().is_empty() {
                            let timeout = Duration::from_secs(2);
                            if let Err(error) =
                                mdns.verify(info.get_fullname().to_string(), timeout)
                            {
                                println!("  verify failed: {error}");
                            } else {
                                println!("  verify started (timeout={}ms)", timeout.as_millis());
                            }
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    return Err("browse receiver disconnected".to_string());
                }
            }
        } else {
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    if let Some(fullname) = registered_fullname {
        println!("Unregistering {fullname}...");
        if let Ok(receiver) = mdns.unregister(&fullname) {
            while let Ok(event) = receiver.recv_timeout(Duration::from_millis(400)) {
                println!("  unregister event: {:?}", event);
            }
        }
    }

    println!("resolved services observed: {resolved_count}");
    mdns.shutdown().map_err(|error| error.to_string())?;
    println!("trekr mDNS probe finished");
    Ok(())
}

fn parse_args() -> Result<ProbeConfig, String> {
    let mut service_type = "_trekr-midi._udp".to_string();
    let mut instance_name = "trekr-mdns-probe".to_string();
    let mut host_name = "trekr-mdns-probe.local.".to_string();
    let mut port = 50040_u16;
    let mut duration_secs = Some(60_u64);
    let mut mode = ProbeMode::Both;
    let mut verify = false;
    let mut disable_ipv6 = false;
    let mut include_apple_p2p = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--service-type" => {
                service_type = args
                    .next()
                    .ok_or_else(|| "missing value for --service-type".to_string())?;
            }
            "--instance" => {
                instance_name = args
                    .next()
                    .ok_or_else(|| "missing value for --instance".to_string())?;
            }
            "--host" => {
                host_name = args
                    .next()
                    .ok_or_else(|| "missing value for --host".to_string())?;
            }
            "--port" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --port".to_string())?;
                port = value
                    .parse::<u16>()
                    .map_err(|_| format!("invalid --port value '{value}'"))?;
            }
            "--duration-secs" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --duration-secs".to_string())?;
                let parsed = value
                    .parse::<u64>()
                    .map_err(|_| format!("invalid --duration-secs value '{value}'"))?;
                duration_secs = (parsed != 0).then_some(parsed);
            }
            "--browse-only" => mode = ProbeMode::BrowseOnly,
            "--advertise-only" => mode = ProbeMode::AdvertiseOnly,
            "--verify" => verify = true,
            "--disable-ipv6" => disable_ipv6 = true,
            "--include-apple-p2p" => include_apple_p2p = true,
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    Ok(ProbeConfig {
        service_type: normalize_service_type(&service_type),
        instance_name,
        host_name: normalize_host_name(&host_name),
        port,
        duration: duration_secs.map(Duration::from_secs),
        mode,
        verify,
        disable_ipv6,
        include_apple_p2p,
    })
}

fn normalize_service_type(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.ends_with(".local.") {
        trimmed.to_string()
    } else if trimmed.ends_with('.') {
        format!("{trimmed}local.")
    } else {
        format!("{trimmed}.local.")
    }
}

fn normalize_host_name(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.ends_with('.') {
        trimmed.to_string()
    } else {
        format!("{trimmed}.")
    }
}

fn print_usage() {
    println!("trekr mDNS capability probe");
    println!();
    println!("Usage:");
    println!("  cargo run --bin trekr-mdns-probe -- [options]");
    println!();
    println!("Options:");
    println!("  --service-type <type>   Service type, default: _trekr-midi._udp");
    println!("  --instance <name>       Instance name, default: trekr-mdns-probe");
    println!("  --host <fqdn>           Host name, default: trekr-mdns-probe.local.");
    println!("  --port <u16>            Service port, default: 50040");
    println!("  --duration-secs <n>     Probe duration, default: 60, 0 = run forever");
    println!("  --browse-only           Only browse");
    println!("  --advertise-only        Only advertise");
    println!("  --verify                Verify resolved services (IPv4)");
    println!("  --disable-ipv6          Disable IPv6 interfaces in mdns daemon");
    println!("  --include-apple-p2p     Include Apple p2p interfaces");
    println!("  -h, --help              Show help");
    println!();
    println!("Examples:");
    println!(
        "  cargo run --bin trekr-mdns-probe -- --service-type _apple-midi._udp --duration-secs 120"
    );
    println!(
        "  cargo run --bin trekr-mdns-probe -- --browse-only --service-type _services._dns-sd._udp"
    );
}
