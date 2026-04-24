use super::*;

pub(crate) fn port_name(port: Option<&MidiPortRef>) -> &str {
    port.map(|value| value.name.as_str()).unwrap_or("none")
}

pub(crate) fn resolve_port_by_name(
    ports: &[MidiPortRef],
    preferred_name: Option<&str>,
) -> Option<usize> {
    let preferred_name = preferred_name?;
    ports.iter().position(|port| port.name == preferred_name)
}

pub(crate) fn clamp_index(index: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        index.min(len - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_name_and_lookup_handle_missing_values() {
        let ports = vec![
            MidiPortRef {
                name: "Input A".to_string(),
            },
            MidiPortRef {
                name: "Input B".to_string(),
            },
        ];
        assert_eq!(port_name(None), "none");
        assert_eq!(port_name(ports.first()), "Input A");
        assert_eq!(resolve_port_by_name(&ports, Some("Input B")), Some(1));
        assert_eq!(resolve_port_by_name(&ports, Some("Missing")), None);
        assert_eq!(resolve_port_by_name(&ports, None), None);
    }

    #[test]
    fn clamp_index_caps_to_last_valid_slot() {
        assert_eq!(clamp_index(4, 0), 0);
        assert_eq!(clamp_index(1, 4), 1);
        assert_eq!(clamp_index(9, 4), 3);
    }
}
