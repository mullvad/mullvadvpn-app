use mullvad_types::{location::GeoIpLocation, states::TunnelState};
use talpid_types::{
    net::{Endpoint, TunnelEndpoint},
    tunnel::ErrorState,
};

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

    match error_state.cause() {
        #[cfg(target_os = "linux")]
        cause @ talpid_types::tunnel::ErrorStateCause::SetFirewallPolicyError(_) => {
            println!("Blocked: {}", cause);
            println!("Your kernel might be terribly out of date or missing nftables");
        }
        cause => println!("Blocked: {}", cause),
    }
}
