use mullvad_management_interface::types::{
    error_state::{
        firewall_policy_error::ErrorType as FirewallPolicyErrorType, Cause as ErrorStateCause,
        FirewallPolicyError, GenerationError,
    },
    tunnel_state,
    tunnel_state::State::*,
    ErrorState, KeygenEvent, ProxyType, TransportProtocol, TunnelEndpoint, TunnelState, TunnelType,
};
use mullvad_types::auth_failed::AuthFailed;
use std::fmt::Write;

pub fn print_keygen_event(key_event: &KeygenEvent) {
    use mullvad_management_interface::types::keygen_event::KeygenEvent as EventType;

    match EventType::from_i32(key_event.event).unwrap() {
        EventType::NewKey => {
            println!(
                "New WireGuard key: {}",
                base64::encode(&key_event.new_key.as_ref().unwrap().key)
            );
        }
        EventType::TooManyKeys => {
            println!("Account has too many keys already");
        }
        EventType::GenerationFailure => {
            println!("Failed to generate new WireGuard key");
        }
    }
}

pub fn print_state(state: &TunnelState) {
    print!("Tunnel status: ");
    match state.state.as_ref().unwrap() {
        Error(error) => print_error_state(error.error_state.as_ref().unwrap()),
        Connected(tunnel_state::Connected { relay_info }) => {
            let endpoint = relay_info
                .as_ref()
                .unwrap()
                .tunnel_endpoint
                .as_ref()
                .unwrap();
            println!("Connected to {}", format_endpoint(&endpoint));
        }
        Connecting(tunnel_state::Connecting { relay_info }) => {
            let endpoint = relay_info
                .as_ref()
                .unwrap()
                .tunnel_endpoint
                .as_ref()
                .unwrap();
            println!("Connecting to {}...", format_endpoint(&endpoint));
        }
        Disconnected(_) => println!("Disconnected"),
        Disconnecting(_) => println!("Disconnecting..."),
    }
}

fn format_endpoint(endpoint: &TunnelEndpoint) -> String {
    let tunnel_type = TunnelType::from_i32(endpoint.tunnel_type).expect("invalid tunnel protocol");
    let mut out = format!(
        "{} {} over {}",
        match tunnel_type {
            TunnelType::Wireguard => "WireGuard",
            TunnelType::Openvpn => "OpenVPN",
        },
        endpoint.address,
        format_protocol(
            TransportProtocol::from_i32(endpoint.protocol).expect("invalid transport protocol")
        ),
    );

    match tunnel_type {
        TunnelType::Openvpn => {
            if let Some(ref proxy) = endpoint.proxy {
                write!(
                    &mut out,
                    " via {} {} over {}",
                    match ProxyType::from_i32(proxy.proxy_type).expect("invalid proxy type") {
                        ProxyType::Shadowsocks => "Shadowsocks",
                        ProxyType::Custom => "custom bridge",
                    },
                    proxy.address,
                    format_protocol(
                        TransportProtocol::from_i32(proxy.protocol)
                            .expect("invalid transport protocol")
                    ),
                )
                .unwrap();
            }
        }
        TunnelType::Wireguard => {
            if let Some(ref entry_endpoint) = endpoint.entry_endpoint {
                write!(
                    &mut out,
                    " via {} over {}",
                    entry_endpoint.address,
                    format_protocol(
                        TransportProtocol::from_i32(entry_endpoint.protocol)
                            .expect("invalid transport protocol")
                    )
                )
                .unwrap();
            }
        }
    }

    out
}

fn print_error_state(error_state: &ErrorState) {
    if error_state.blocking_error.is_some() {
        eprintln!("Mullvad daemon failed to setup firewall rules!");
        eprintln!("Deamon cannot block traffic from flowing, non-local traffic will leak");
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
        #[cfg(target_os = "macos")]
        CustomResolverError => "Failed to start custom resolver",
        #[cfg(target_os = "macos")]
        ReadSystemDnsConfig => "Failed to read system DNS config",
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
