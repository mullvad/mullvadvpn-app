use std::collections::HashMap;

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
    match state {
        Disconnected {
            location,
            locked_down,
        } => {
            let old_location = match previous_state {
                Some(Disconnected {
                    location,
                    locked_down: was_locked_down,
                }) => {
                    if *locked_down && !was_locked_down {
                        print_option!("Internet access is blocked due to lockdown mode");
                    } else if !*locked_down && *was_locked_down {
                        print_option!("Internet access is no longer blocked due to lockdown mode");
                    }
                    location
                }
                _ => {
                    println!("Disconnected");
                    if *locked_down {
                        print_option!("Internet access is blocked due to lockdown mode");
                    }
                    &None
                }
            };
            let location_fmt = location.as_ref().map(format_location).unwrap_or_default();
            let old_location_fmt = old_location
                .as_ref()
                .map(format_location)
                .unwrap_or_default();
            if location_fmt != old_location_fmt {
                print_option!("Visible location", location_fmt);
            }
        }
        Connecting {
            endpoint,
            location,
            feature_indicators,
        } => {
            let (old_endpoint, old_location, old_feature_indicators) = match previous_state {
                Some(Connecting {
                    endpoint,
                    location,
                    feature_indicators,
                }) => (Some(endpoint), location, Some(feature_indicators)),
                _ => {
                    println!("Connecting");
                    (None, &None, None)
                }
            };

            print_relay_info(
                endpoint,
                old_endpoint,
                location.as_ref(),
                old_location.as_ref(),
                feature_indicators,
                old_feature_indicators,
                verbose,
            );
        }
        Connected {
            endpoint,
            location,
            feature_indicators,
        } => {
            let (old_endpoint, old_location, old_feature_indicators) = match previous_state {
                Some(Connected {
                    endpoint,
                    location,
                    feature_indicators,
                }) => (Some(endpoint), location, Some(feature_indicators)),
                Some(Connecting {
                    endpoint,
                    location,
                    feature_indicators,
                }) => {
                    println!("Connected");
                    (Some(endpoint), location, Some(feature_indicators))
                }
                _ => {
                    println!("Connected");
                    (None, &None, None)
                }
            };

            print_relay_info(
                endpoint,
                old_endpoint,
                location.as_ref(),
                old_location.as_ref(),
                feature_indicators,
                old_feature_indicators,
                verbose,
            );
        }
        Disconnecting(ActionAfterDisconnect::Reconnect) => {}
        Disconnecting(_) => println!("Disconnecting"),
        Error(e) => print_error_state(e),
    }
}

fn connection_information(
    endpoint: Option<&TunnelEndpoint>,
    location: Option<&GeoIpLocation>,
    feature_indicators: Option<&FeatureIndicators>,
    verbose: bool,
) -> HashMap<&'static str, String> {
    let mut info = HashMap::new();
    if let Some(endpoint) = endpoint {
        info.insert(
            "Relay",
            format_relay_connection(endpoint, location, verbose),
        );
        if verbose {
            if let Some(tunnel_interface) = endpoint.tunnel_interface.clone() {
                info.insert("Tunnel interface", tunnel_interface);
            }
            if let Some(bridge) = &endpoint.proxy {
                info.insert("Bridge type", bridge.proxy_type.to_string());
            }
            info.insert("Tunnel type:", endpoint.tunnel_type.to_string());
        }
    }
    if let Some(location) = location {
        info.insert("Visible location", format_location(location));
    }
    if let Some(feature_indicators) = feature_indicators {
        if !feature_indicators.is_empty() {
            info.insert("Features", feature_indicators.to_string());
        }
    }
    info
}

fn print_relay_info(
    endpoint: &TunnelEndpoint,
    old_endpoint: Option<&TunnelEndpoint>,
    location: Option<&GeoIpLocation>,
    old_location: Option<&GeoIpLocation>,
    feature_indicators: &FeatureIndicators,
    old_feature_indicators: Option<&FeatureIndicators>,
    verbose: bool,
) {
    let current_info =
        connection_information(Some(endpoint), location, Some(feature_indicators), verbose);
    let previous_info =
        connection_information(old_endpoint, old_location, old_feature_indicators, verbose);
    for (name, value) in current_info {
        let previous_value = previous_info.get(name);
        match (value, previous_value) {
            (value, Some(previous_value)) if &value == previous_value => {}
            (value, Some(_previous_value)) => {
                // TODO: use print_option
                println!("{:<4}{:<24}{}", "", format!("{name} (updated):"), value);
            }
            (value, None) => println!("{:<4}{:<24}{}", "", format!("{name}:"), value),
        }
    }
}

pub fn format_location(location: &GeoIpLocation) -> String {
    let mut formatted_location = location.country.to_string();
    if let Some(city) = &location.city {
        formatted_location.push_str(&format!(", {}", city));
    }
    if let Some(ipv4) = location.ipv4 {
        formatted_location.push_str(&format!(". IPv4: {}", ipv4));
    }
    if let Some(ipv6) = location.ipv6 {
        formatted_location.push_str(&format!(", IPv6: {}", ipv6));
    }
    formatted_location
}

fn format_relay_connection(
    endpoint: &TunnelEndpoint,
    location: Option<&GeoIpLocation>,
    verbose: bool,
) -> String {
    let exit_endpoint = format_endpoint(
        location.and_then(|l| l.hostname.as_deref()),
        &endpoint.endpoint,
        verbose,
    );

    let first_hop = endpoint.entry_endpoint.as_ref().map(|entry| {
        let endpoint = format_endpoint(
            location.and_then(|l| l.entry_hostname.as_deref()),
            entry,
            verbose,
        );
        format!(" via {endpoint}")
    });

    let bridge = endpoint.proxy.as_ref().map(|proxy| {
        let proxy_endpoint = format_endpoint(
            location.and_then(|l| l.bridge_hostname.as_deref()),
            &proxy.endpoint,
            verbose,
        );

        format!(" via {proxy_endpoint}")
    });

    format!(
        "{exit_endpoint}{first_hop}{bridge}",
        first_hop = first_hop.unwrap_or_default(),
        bridge = bridge.unwrap_or_default(),
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
