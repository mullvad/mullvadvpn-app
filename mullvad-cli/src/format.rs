use itertools::Itertools;
use mullvad_types::{
    auth_failed::AuthFailed, features::FeatureIndicators, location::GeoIpLocation,
    states::TunnelState,
};
use talpid_types::{
    net::{Endpoint, TunnelEndpoint},
    tunnel::{ActionAfterDisconnect, ErrorState},
};

#[macro_export]
macro_rules! print_option {
    ($value:expr $(,)?) => {{
        println!("{:<4}{:<24}{}", "", "", $value,)
    }};
    ($option:expr, $value:expr $(,)?) => {{
        println!("{:<4}{:<24}{}", "", concat!($option, ":"), $value,)
    }};
}

pub fn print_state(state: &TunnelState, previous_state: Option<&TunnelState>, verbose: bool) {
    use TunnelState::*;

    // When we enter the connected or disconnected state, am.i.mullvad.net will
    // be polled to get exit location. When it arrives, we will get another
    // tunnel state of the same enum type, but with the location filled in. This
    // match statement checks if the new state is an updated version of the old
    // one and if so skips the print to avoid spamming the user. Note that for
    // graphical frontends updating the drawn state with an identical one is
    // invisible, so this is only an issue for the CLI.
    match (&previous_state, &state) {
        (_, TunnelState::Error(e)) => print_error_state(e),
        (
            Some(TunnelState::Disconnected {
                location: old_location,
                locked_down: was_locked_down,
            }),
            TunnelState::Disconnected {
                location,
                locked_down,
            },
            // Do print an updated state if the lockdown setting was changed
        ) => {
            if *locked_down && !was_locked_down {
                println!("  Internet access is blocked due to lockdown mode");
            } else if !*locked_down && *was_locked_down {
                println!("  Internet access is no longer blocked due to lockdown mode");
            }

            if location != old_location {
                if let Some(location) = location {
                    print!("  ");
                    print_location(location); // TODO: look over
                }
            }
        }
        (
            _,
            TunnelState::Disconnected {
                location,
                locked_down,
            },
            // Do print an updated state if the lockdown setting was changed
        ) => {
            println!("Disconnected");
            if *locked_down {
                println!("Internet access is blocked due to lockdown mode");
            }
            if let Some(location) = location {
                print!("  ");
                print_location(location);
            }
        }
        (
            old @ Some(TunnelState::Connecting {
                feature_indicators: old_feature_indicators,
                endpoint: _,
                location: old_location,
            })
            | old @ Some(TunnelState::Connected {
                feature_indicators: old_feature_indicators,
                location: old_location,
                endpoint: _,
            }),
            TunnelState::Connected {
                feature_indicators,
                location,
                endpoint,
            },
        ) => {
            if let None | Some(&Connecting { .. }) = old {
                println!("Connected");
            }
            if feature_indicators != old_feature_indicators {
                print!("  ");
                println!(
                    "Updated features: {}",
                    format_feature_indicators(feature_indicators)
                );
            }

            if location != old_location {
                if let Some(location) = location {
                    print!("  ");
                    print_location(location);
                }
            }
            if verbose {
                if let Some(tunnel_interface) = &endpoint.tunnel_interface {
                    print!("  ");
                    println!("Tunnel interface: {tunnel_interface}")
                }
            }
        }
        (_, Disconnecting(ActionAfterDisconnect::Reconnect)) => {}
        (_, Disconnecting(_)) => {
            println!("Disconnecting")
        }
        (
            None,
            TunnelState::Connected {
                endpoint,
                location,
                feature_indicators,
            },
            // Do print an updated state if the feature indicators changed
        ) => {
            println!(
                "Connected to {}",
                format_relay_connection(endpoint, location.as_ref(), verbose)
            );
            let features = format_feature_indicators(feature_indicators);
            if !features.is_empty() {
                print!("  ");
                println!("Active features: {}", features);
            }

            if verbose {
                if let Some(tunnel_interface) = &endpoint.tunnel_interface {
                    print!("  ");
                    println!("Tunnel interface: {tunnel_interface}")
                }
            }
        }
        (
            Some(TunnelState::Connecting {
                feature_indicators: old_feature_indicators,
                endpoint: old_endpoint,
                location: old_location,
            }),
            TunnelState::Connecting {
                feature_indicators,
                endpoint,
                location,
            },
            // Do print an updated state if the feature indicators changed
        ) => {
            if endpoint != old_endpoint || location != old_location {
                println!(
                    "New connection attempt to {}...",
                    format_relay_connection(state.endpoint().unwrap(), location.as_ref(), verbose)
                );
            }

            if feature_indicators != old_feature_indicators {
                print!("  ");
                println!(
                    "Features: {}",
                    format_feature_indicators(feature_indicators)
                );
            }
            if verbose {
                print!("  ");
                if let Some(tunnel_interface) = &state.endpoint().unwrap().tunnel_interface {
                    println!("Tunnel interface: {tunnel_interface}")
                }
            }
        }
        (
            _,
            Connecting {
                endpoint,
                location,
                feature_indicators,
            },
            // Do print an updated state if the feature indicators changed
        ) => {
            println!(
                "Connecting to {}...",
                format_relay_connection(endpoint, location.as_ref(), verbose)
            );
            let features = format_feature_indicators(feature_indicators);
            if !features.is_empty() {
                print!("  ");
                println!("Features: {}", features);
            }
            if verbose {
                if let Some(tunnel_interface) = &endpoint.tunnel_interface {
                    print!("  ");
                    println!("Tunnel interface: {tunnel_interface}")
                }
            }
        }
        (old, new) => unreachable!("Transition from {old:?} to {new:?} not expected"),
    }
}

pub fn print_location(location: &GeoIpLocation) {
    print!("Your connection appears to be from: {}", location.country);
    if let Some(city) = &location.city {
        print!(", {}", city);
    }
    if let Some(ipv4) = location.ipv4 {
        print!(". IPv4: {ipv4}");
    }
    if let Some(ipv6) = location.ipv6 {
        print!(", IPv6: {ipv6}");
    }
    println!();
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
                format!("{exit} in {city}, {country}")
            }
            Some(GeoIpLocation {
                country,
                city: None,
                ..
            }) => {
                format!("{exit} in {country}")
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

    let mut bridge_type = String::new();
    if verbose {
        if let Some(bridge) = &endpoint.proxy {
            bridge_type = format!("\nBridge type: {}", bridge.proxy_type);
        }
    }

    format!(
        "{exit_endpoint}{first_hop}{bridge}{obfuscator}{tunnel_type}{bridge_type}",
        first_hop = first_hop.unwrap_or_default(),
        bridge = bridge.unwrap_or_default(),
        obfuscator = obfuscator.unwrap_or_default(),
    )
}

fn format_feature_indicators(feature_indicators: &FeatureIndicators) -> String {
    feature_indicators
        .active_features()
        // Sort the features alphabetically (Just to have some order, arbitrarily chosen)
        .sorted_by_key(|feature| feature.to_string())
        .join(", ")
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
            println!("Blocked: {cause}");
            println!("Your kernel might be terribly out of date or missing nftables");
        }
        #[cfg(target_os = "macos")]
        cause @ talpid_types::tunnel::ErrorStateCause::NeedFullDiskPermissions => {
            println!("Blocked: {cause}");
            println!();
            println!(
                r#"Enable "Full Disk Access" for "Mullvad VPN" in the macOS system settings:"#
            );
            println!(
                r#"open "x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles""#
            );
            println!();
            println!("Restart the Mullvad daemon for the change to take effect:");
            println!("launchctl unload -w /Library/LaunchDaemons/net.mullvad.daemon.plist");
            println!("launchctl load -w /Library/LaunchDaemons/net.mullvad.daemon.plist");
        }
        talpid_types::tunnel::ErrorStateCause::AuthFailed(Some(auth_failed)) => {
            println!(
                "Blocked: Authentication with remote server failed: {}",
                get_auth_failed_message(AuthFailed::from(auth_failed.as_str()))
            );
        }
        cause => println!("Blocked: {cause}"),
    }
}

const fn get_auth_failed_message(auth_failed: AuthFailed) -> &'static str {
    const INVALID_ACCOUNT_MSG: &str = "You've logged in with an account number that is not valid. Please log out and try another one.";
    const EXPIRED_ACCOUNT_MSG: &str = "You have no more VPN time left on this account. Please log in on our website to buy more credit.";
    const TOO_MANY_CONNECTIONS_MSG: &str = "This account has too many simultaneous connections. Disconnect another device or try connecting again shortly.";
    const UNKNOWN_MSG: &str = "Unknown error.";

    match auth_failed {
        AuthFailed::InvalidAccount => INVALID_ACCOUNT_MSG,
        AuthFailed::ExpiredAccount => EXPIRED_ACCOUNT_MSG,
        AuthFailed::TooManyConnections => TOO_MANY_CONNECTIONS_MSG,
        AuthFailed::Unknown => UNKNOWN_MSG,
    }
}
