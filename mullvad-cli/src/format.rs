use mullvad_management_interface::types::{
    error_state::{
        firewall_policy_error::ErrorType as FirewallPolicyErrorType, Cause as ErrorStateCause,
        FirewallPolicyError, GenerationError,
    },
    tunnel_state,
    tunnel_state::State::*,
    ErrorState, ObfuscationType, ProxyType, TransportProtocol, TunnelState, TunnelStateRelayInfo,
    TunnelType,
};
use mullvad_types::auth_failed::AuthFailed;

pub fn print_state(state: &TunnelState, verbose: bool) {
    match state.state.as_ref().unwrap() {
        Error(error) => print_error_state(error.error_state.as_ref().unwrap()),
        Connected(tunnel_state::Connected { relay_info }) => {
            println!(
                "Connected to {}",
                format_relay_connection(relay_info.as_ref().unwrap(), verbose)
            );
        }
        Connecting(tunnel_state::Connecting { relay_info }) => {
            let ellipsis = if !verbose { "..." } else { "" };
            println!(
                "Connecting to {}{ellipsis}",
                format_relay_connection(relay_info.as_ref().unwrap(), verbose)
            );
        }
        Disconnected(_) => println!("Disconnected"),
        Disconnecting(_) => println!("Disconnecting..."),
    }
}

fn format_relay_connection(relay_info: &TunnelStateRelayInfo, verbose: bool) -> String {
    let endpoint = relay_info.tunnel_endpoint.as_ref().unwrap();
    let location = &relay_info.location.as_ref().unwrap();

    let prefix_separator = if verbose { "\n\t" } else { " " };
    let mut obfuscator_overlaps = false;

    let exit_endpoint = {
        let mut address = endpoint.address.as_str();
        let mut protocol = endpoint.protocol;
        if let Some(obfuscator) = endpoint.obfuscation.as_ref() {
            if location.hostname == location.obfuscator_hostname {
                obfuscator_overlaps = true;
                address = &obfuscator.address;
                protocol = obfuscator.protocol;
            }
        };

        let exit = format_endpoint(
            &location.hostname,
            protocol,
            Some(address).filter(|_| verbose),
        );
        format!("{exit} in {}, {}", &location.city, &location.country)
    };

    let first_hop = endpoint.entry_endpoint.as_ref().map(|entry| {
        let mut address = entry.address.as_str();
        let mut protocol = entry.protocol;
        if let Some(obfuscator) = endpoint.obfuscation.as_ref() {
            obfuscator_overlaps = true;
            if location.entry_hostname == location.obfuscator_hostname {
                address = &obfuscator.address;
                protocol = obfuscator.protocol;
            }
        };

        let endpoint = format_endpoint(
            &location.entry_hostname,
            protocol,
            Some(address).filter(|_| verbose),
        );
        format!("{prefix_separator}via {endpoint}")
    });

    let obfuscator = endpoint.obfuscation.as_ref().map(|obfuscator| {
        if !obfuscator_overlaps {
            let endpoint_str = format_endpoint(
                &location.obfuscator_hostname,
                obfuscator.protocol,
                Some(obfuscator.address.as_str()).filter(|_| verbose),
            );
            format!("{prefix_separator}obfuscated via {endpoint_str}")
        } else {
            String::new()
        }
    });

    let bridge = endpoint.proxy.as_ref().map(|proxy| {
        let proxy_endpoint = format_endpoint(
            &location.bridge_hostname,
            proxy.protocol,
            Some(proxy.address.as_str()).filter(|_| verbose),
        );

        format!("{prefix_separator}via {proxy_endpoint}")
    });
    let tunnel_type = if verbose {
        let tunnel = match TunnelType::from_i32(endpoint.tunnel_type).expect("invalid tunnel type")
        {
            TunnelType::Wireguard => "WireGuard",
            TunnelType::Openvpn => "OpenVPN",
        };

        format!("\nTunnel type: {tunnel}")
    } else {
        String::new()
    };
    let quantum_resistant = if !verbose {
        ""
    } else if endpoint.quantum_resistant {
        "\nQuantum resistant tunnel: yes"
    } else {
        "\nQuantum resistant tunnel: no"
    };

    let mut bridge_type = String::new();
    let mut obfuscator_type = String::new();
    if verbose {
        if let Some(bridge) = endpoint.proxy.as_ref() {
            let bridge = match ProxyType::from_i32(bridge.proxy_type).expect("invalid proxy type") {
                ProxyType::Shadowsocks => "Shadowsocks",
                ProxyType::Custom => "custom bridge",
            };
            bridge_type = format!("\nBridge type: {}", bridge);
        }
        if let Some(obfuscator) = endpoint.obfuscation.as_ref() {
            let obfuscation = convert_obfuscator_type(obfuscator.obfuscation_type);
            obfuscator_type = format!("\nObfuscator: {obfuscation}");
        }
    }

    format!(
        "{exit_endpoint}{first_hop}{bridge}{obfuscator}{tunnel_type}{quantum_resistant}{bridge_type}{obfuscator_type}",
        first_hop = first_hop.unwrap_or_default(),
        bridge = bridge.unwrap_or_default(),
        obfuscator = obfuscator.unwrap_or_default(),
    )
}

fn convert_obfuscator_type(obfuscator: i32) -> &'static str {
    match ObfuscationType::from_i32(obfuscator).expect("invalid obfuscator type") {
        ObfuscationType::Udp2tcp => "Udp2Tcp",
    }
}

fn format_endpoint(hostname: &String, protocol_enum: i32, addr: Option<&str>) -> String {
    let protocol = format_protocol(
        TransportProtocol::from_i32(protocol_enum).expect("invalid transport protocol"),
    );
    let sockaddr_suffix = addr
        .map(|addr| format!(" ({addr}/{protocol})"))
        .unwrap_or_else(String::new);
    format!("{hostname}{sockaddr_suffix}")
}

fn print_error_state(error_state: &ErrorState) {
    if error_state.blocking_error.is_some() {
        eprintln!("Mullvad daemon failed to setup firewall rules!");
        eprintln!("Daemon cannot block traffic from flowing, non-local traffic will leak");
    }

    match ErrorStateCause::from_i32(error_state.cause) {
        Some(ErrorStateCause::AuthFailed) => {
            println!(
                "Blocked: {}",
                AuthFailed::from(error_state.auth_fail_reason.as_ref())
            );
        }
        #[cfg(target_os = "linux")]
        Some(ErrorStateCause::SetFirewallPolicyError) => {
            println!("Blocked: {}", error_state_to_string(error_state));
            println!("Your kernel might be terribly out of date or missing nftables");
        }
        _ => println!("Blocked: {}", error_state_to_string(error_state)),
    }
}

fn error_state_to_string(error_state: &ErrorState) -> String {
    use ErrorStateCause::*;

    let error_str = match ErrorStateCause::from_i32(error_state.cause).expect("unknown error cause")
    {
        AuthFailed => {
            return if error_state.auth_fail_reason.is_empty() {
                "Authentication with remote server failed".to_string()
            } else {
                format!(
                    "Authentication with remote server failed: {}",
                    error_state.auth_fail_reason
                )
            };
        }
        Ipv6Unavailable => "Failed to configure IPv6 because it's disabled in the platform",
        SetFirewallPolicyError => {
            return policy_error_to_string(error_state.policy_error.as_ref().unwrap())
        }
        SetDnsError => "Failed to set system DNS server",
        StartTunnelError => "Failed to start connection to remote server",
        TunnelParameterError => {
            return format!(
                "Failure to generate tunnel parameters: {}",
                tunnel_parameter_error_to_string(error_state.parameter_error)
            );
        }
        IsOffline => "This device is offline, no tunnels can be established",
        #[cfg(target_os = "android")]
        VpnPermissionDenied => "The Android VPN permission was denied when creating the tunnel",
        #[cfg(target_os = "windows")]
        SplitTunnelError => "The split tunneling module reported an error",
        #[cfg(not(target_os = "android"))]
        _ => unreachable!("unknown error cause"),
    };

    error_str.to_string()
}

fn tunnel_parameter_error_to_string(parameter_error: i32) -> &'static str {
    match GenerationError::from_i32(parameter_error).expect("unknown generation error") {
        GenerationError::NoMatchingRelay => "Failure to select a matching tunnel relay",
        GenerationError::NoMatchingBridgeRelay => "Failure to select a matching bridge relay",
        GenerationError::NoWireguardKey => "No wireguard key available",
        GenerationError::CustomTunnelHostResolutionError => {
            "Can't resolve hostname for custom tunnel host"
        }
    }
}

fn policy_error_to_string(policy_error: &FirewallPolicyError) -> String {
    let cause = match FirewallPolicyErrorType::from_i32(policy_error.r#type)
        .expect("unknown policy error")
    {
        FirewallPolicyErrorType::Generic => return "Failed to set firewall policy".to_string(),
        FirewallPolicyErrorType::Locked => format!(
            "An application prevented the firewall policy from being set: {} (pid {})",
            policy_error.lock_name, policy_error.lock_pid
        ),
    };
    format!("Failed to set firewall policy: {}", cause)
}

fn format_protocol(protocol: TransportProtocol) -> &'static str {
    match protocol {
        TransportProtocol::Udp => "UDP",
        TransportProtocol::Tcp => "TCP",
    }
}
