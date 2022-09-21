use mullvad_management_interface::types::{
    error_state::{
        firewall_policy_error::ErrorType as FirewallPolicyErrorType, AuthFailedError,
        Cause as ErrorStateCause, FirewallPolicyError, GenerationError,
    },
    tunnel_state,
    tunnel_state::State::*,
    ErrorState, ObfuscationType, ProxyType, TransportProtocol, TunnelState, TunnelStateRelayInfo,
    TunnelType,
};
use std::borrow::Cow;

pub fn print_state(state: &TunnelState, verbose: bool) {
    use TunnelState::*;

    match state {
        Error(error) => print_error_state(error),
        Connected { endpoint, location } => {
            println!(
                "Connected to {}",
                format_relay_connection(endpoint, location.as_ref(), verbose)
            );
        }
        Connecting { endpoint, location } => {
            let ellipsis = if !verbose { "..." } else { "" };
            println!(
                "Connecting to {}{ellipsis}",
                format_relay_connection(endpoint, location.as_ref(), verbose)
            );
        }
        Disconnected => println!("Disconnected"),
        Disconnecting(_) => println!("Disconnecting..."),
    }
}

fn format_relay_connection(
    endpoint: &TunnelEndpoint,
    location: Option<&GeoIpLocation>,
    verbose: bool,
) -> String {
    let prefix_separator = if verbose { "\n\t" } else { " " };
    let mut obfuscator_overlaps = false;

    let exit_endpoint = {
        let mut exit_endpoint = &endpoint.endpoint;
        if let Some(obfuscator) = &endpoint.obfuscation {
            if location
                .map(|l| l.hostname == l.obfuscator_hostname)
                .unwrap_or(false)
            {
                obfuscator_overlaps = true;
                exit_endpoint = &obfuscator.endpoint;
            }
        };

        let exit = format_endpoint(
            location.and_then(|l| l.hostname.as_deref()),
            exit_endpoint,
            verbose,
        );
        match location {
            Some(GeoIpLocation {
                country,
                city: Some(city),
                ..
            }) => {
                format!("{exit} in {}, {}", city, country)
            }
            Some(GeoIpLocation {
                country,
                city: None,
                ..
            }) => {
                format!("{exit} in {}", country)
            }
            None => exit,
        }
    };

    let first_hop = endpoint.entry_endpoint.as_ref().map(|entry| {
        let mut entry_endpoint = entry;
        if let Some(obfuscator) = &endpoint.obfuscation {
            if location
                .map(|l| l.entry_hostname == l.obfuscator_hostname)
                .unwrap_or(false)
            {
                obfuscator_overlaps = true;
                entry_endpoint = &obfuscator.endpoint;
            }
        };

        let endpoint = format_endpoint(
            location.and_then(|l| l.entry_hostname.as_deref()),
            entry_endpoint,
            verbose,
        );
        format!("{prefix_separator}via {endpoint}")
    });

    let obfuscator = endpoint.obfuscation.as_ref().map(|obfuscator| {
        if !obfuscator_overlaps {
            let endpoint_str = format_endpoint(
                location.and_then(|l| l.obfuscator_hostname.as_deref()),
                &obfuscator.endpoint,
                verbose,
            );
            format!("{prefix_separator}obfuscated via {endpoint_str}")
        } else {
            String::new()
        }
    });

    let bridge = endpoint.proxy.as_ref().map(|proxy| {
        let proxy_endpoint = format_endpoint(
            location.and_then(|l| l.bridge_hostname.as_deref()),
            &proxy.endpoint,
            verbose,
        );

        format!("{prefix_separator}via {proxy_endpoint}")
    });
    let tunnel_type = if verbose {
        format!("\nTunnel type: {}", endpoint.tunnel_type)
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
        if let Some(bridge) = &endpoint.proxy {
            bridge_type = format!("\nBridge type: {}", bridge.proxy_type);
        }
        if let Some(obfuscator) = &endpoint.obfuscation {
            obfuscator_type = format!("\nObfuscator: {}", obfuscator.obfuscation_type);
        }
    }

    format!(
        "{exit_endpoint}{first_hop}{bridge}{obfuscator}{tunnel_type}{quantum_resistant}{bridge_type}{obfuscator_type}",
        first_hop = first_hop.unwrap_or_default(),
        bridge = bridge.unwrap_or_default(),
        obfuscator = obfuscator.unwrap_or_default(),
    )
}

fn format_endpoint(hostname: Option<&str>, endpoint: &Endpoint, verbose: bool) -> String {
    match (hostname, verbose) {
        (Some(hostname), true) => format!("{hostname} ({endpoint})"),
        (None, true) => endpoint.to_string(),
        (Some(hostname), false) => hostname.to_string(),
        (None, false) => endpoint.address.to_string(),
    }
}

fn print_error_state(error_state: &ErrorState) {
    if error_state.block_failure().is_some() {
        eprintln!("Mullvad daemon failed to setup firewall rules!");
        eprintln!("Daemon cannot block traffic from flowing, non-local traffic will leak");
    }

    match ErrorStateCause::from_i32(error_state.cause) {
        Some(ErrorStateCause::AuthFailed) => {
            println!(
                "Blocked: {}",
                auth_failed_error_to_string(error_state.auth_failed_error),
            );
        }
        #[cfg(target_os = "linux")]
        cause @ talpid_types::tunnel::ErrorStateCause::SetFirewallPolicyError(_) => {
            println!("Blocked: {}", cause);
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
            return format!(
                "Authentication with remote server failed: {}",
                auth_failed_error_to_string(error_state.auth_failed_error)
            );
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

fn auth_failed_error_to_string(auth_failed_error: i32) -> &'static str {
    const INVALID_ACCOUNT_MSG: &str = "You've logged in with an account number that is not valid. Please log out and try another one.";
    const EXPIRED_ACCOUNT_MSG: &str = "You have no more VPN time left on this account. Please log in on our website to buy more credit.";
    const TOO_MANY_CONNECTIONS_MSG: &str = "This account has too many simultaneous connections. Disconnect another device or try connecting again shortly.";
    const UNKNOWN_MSG: &str = "Unknown error.";

    match AuthFailedError::from_i32(auth_failed_error).expect("invalid auth failed error") {
        AuthFailedError::InvalidAccount => INVALID_ACCOUNT_MSG,
        AuthFailedError::ExpiredAccount => EXPIRED_ACCOUNT_MSG,
        AuthFailedError::TooManyConnections => TOO_MANY_CONNECTIONS_MSG,
        AuthFailedError::Unknown => UNKNOWN_MSG,
    }
}

fn format_protocol(protocol: TransportProtocol) -> &'static str {
    match protocol {
        TransportProtocol::Udp => "UDP",
        TransportProtocol::Tcp => "TCP",
    }
}
