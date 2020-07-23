#![cfg(feature = "integration-tests")]

use futures::{stream::Stream, Future};
use mullvad_tests::{
    mock_openvpn::search_openvpn_args, watch_event, DaemonRunner, MockOpenVpnPluginRpcClient,
    PathWatcher,
};
use mullvad_types::{location::GeoIpLocation, states::TunnelState, DaemonEvent};
use std::{fs, path::Path, time::Duration};
use talpid_types::{
    net::{Endpoint, TransportProtocol, TunnelEndpoint, TunnelType},
    tunnel::ActionAfterDisconnect,
};

#[cfg(target_os = "linux")]
const OPENVPN_PLUGIN_NAME: &str = "libtalpid_openvpn_plugin.so";

#[cfg(windows)]
const OPENVPN_PLUGIN_NAME: &str = "talpid_openvpn_plugin.dll";

#[test]
fn spawns_openvpn() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let openvpn_args_file = daemon.mock_openvpn_args_file();
    let mut openvpn_args_file_events = PathWatcher::watch(&openvpn_args_file).unwrap();

    assert!(!openvpn_args_file.exists());

    rpc_client.set_account(Some("123456".to_owned())).unwrap();
    rpc_client.connect().unwrap();

    openvpn_args_file_events.assert_create_write_close_sequence();
}

#[test]
fn respawns_openvpn_if_it_crashes() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let openvpn_args_file = daemon.mock_openvpn_args_file();
    let mut openvpn_args_file_events = PathWatcher::watch(&openvpn_args_file).unwrap();

    openvpn_args_file_events.set_timeout(Duration::from_secs(10));

    assert!(!openvpn_args_file.exists());

    rpc_client.set_account(Some("123456".to_owned())).unwrap();
    rpc_client.connect().unwrap();

    openvpn_args_file_events.assert_create_write_close_sequence();

    // Stop OpenVPN by removing the mock OpenVPN arguments file
    fs::remove_file(&openvpn_args_file).expect("Failed to remove the mock OpenVPN arguments file");
    assert_eq!(openvpn_args_file_events.next(), Some(watch_event::REMOVE));

    openvpn_args_file_events.assert_create_write_close_sequence();
}

#[test]
fn changes_to_connecting_state() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let state_events = rpc_client.daemon_event_subscribe().wait().unwrap();

    rpc_client.set_account(Some("123456".to_owned())).unwrap();
    rpc_client.connect().unwrap();

    let _ = assert_state_event(
        state_events,
        TunnelState::Connecting {
            endpoint: get_default_endpoint(),
            location: get_default_location(),
        },
    );
    assert_eq!(
        rpc_client.get_state().unwrap(),
        TunnelState::Connecting {
            endpoint: get_default_endpoint(),
            location: get_default_location(),
        },
    );
}

#[test]
fn changes_to_connected_state() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let openvpn_args_file = daemon.mock_openvpn_args_file();
    let mut openvpn_args_file_events = PathWatcher::watch(&openvpn_args_file).unwrap();
    let state_events = rpc_client.daemon_event_subscribe().wait().unwrap();

    rpc_client.set_account(Some("123456".to_owned())).unwrap();
    rpc_client.connect().unwrap();

    let state_events = assert_state_event(
        state_events,
        TunnelState::Connecting {
            endpoint: get_default_endpoint(),
            location: get_default_location(),
        },
    );
    openvpn_args_file_events.assert_create_write_close_sequence();

    let mut mock_plugin_client = create_mock_openvpn_plugin_client(openvpn_args_file);

    mock_plugin_client.up().unwrap();

    assert_state_event(
        state_events,
        TunnelState::Connected {
            endpoint: get_default_endpoint(),
            location: get_default_location(),
        },
    );
    assert_eq!(
        rpc_client.get_state().unwrap(),
        TunnelState::Connected {
            endpoint: get_default_endpoint(),
            location: get_default_location(),
        }
    );
}

#[test]
fn returns_to_connecting_state() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let openvpn_args_file = daemon.mock_openvpn_args_file();
    let mut openvpn_args_file_events = PathWatcher::watch(&openvpn_args_file).unwrap();
    let state_events = rpc_client.daemon_event_subscribe().wait().unwrap();

    rpc_client.set_account(Some("123456".to_owned())).unwrap();
    rpc_client.connect().unwrap();

    let state_events = assert_state_event(
        state_events,
        TunnelState::Connecting {
            endpoint: get_default_endpoint(),
            location: get_default_location(),
        },
    );
    openvpn_args_file_events.assert_create_write_close_sequence();

    let mut mock_plugin_client = create_mock_openvpn_plugin_client(openvpn_args_file);

    mock_plugin_client.up().unwrap();

    let state_events = assert_state_event(
        state_events,
        TunnelState::Connected {
            endpoint: get_default_endpoint(),
            location: get_default_location(),
        },
    );

    mock_plugin_client.route_predown().unwrap();

    // Wait for new OpenVPN instance
    assert_eq!(openvpn_args_file_events.next(), Some(watch_event::REMOVE));
    openvpn_args_file_events.assert_create_write_close_sequence();

    let _ = assert_state_event(
        state_events,
        TunnelState::Disconnecting(ActionAfterDisconnect::Reconnect),
    );
}

#[test]
fn disconnects() {
    let mut daemon = DaemonRunner::spawn();
    let mut rpc_client = daemon.rpc_client().unwrap();
    let openvpn_args_file = daemon.mock_openvpn_args_file();
    let mut openvpn_args_file_events = PathWatcher::watch(&openvpn_args_file).unwrap();
    let state_events = rpc_client.daemon_event_subscribe().wait().unwrap();

    rpc_client.set_account(Some("123456".to_owned())).unwrap();
    rpc_client.connect().unwrap();

    let state_events = assert_state_event(
        state_events,
        TunnelState::Connecting {
            endpoint: get_default_endpoint(),
            location: get_default_location(),
        },
    );
    openvpn_args_file_events.assert_create_write_close_sequence();

    let mut mock_plugin_client = create_mock_openvpn_plugin_client(openvpn_args_file);

    mock_plugin_client.up().unwrap();

    let state_events = assert_state_event(
        state_events,
        TunnelState::Connected {
            endpoint: get_default_endpoint(),
            location: get_default_location(),
        },
    );

    rpc_client.disconnect().unwrap();

    let state_events = assert_state_event(
        state_events,
        TunnelState::Disconnecting(ActionAfterDisconnect::Nothing),
    );
    let _ = assert_state_event(state_events, TunnelState::Disconnected);
}

fn get_default_endpoint() -> TunnelEndpoint {
    TunnelEndpoint {
        endpoint: Endpoint {
            address: "192.168.0.100:1000".parse().unwrap(),
            protocol: TransportProtocol::Udp,
        },
        tunnel_type: TunnelType::OpenVpn,
        proxy: None,
    }
}

fn get_default_location() -> Option<GeoIpLocation> {
    Some(GeoIpLocation {
        ipv4: None,
        ipv6: None,
        country: "Sweden".to_string(),
        city: Some("Gothenburg".to_string()),
        latitude: 57.70887,
        longitude: 11.97456,
        mullvad_exit_ip: true,
        hostname: Some("fakehost".to_string()),
        bridge_hostname: None,
    })
}

fn assert_state_event<
    S: Stream<Item = DaemonEvent, Error = jsonrpc_client_core::Error> + std::fmt::Debug,
>(
    mut receiver: S,
    expected_state: TunnelState,
) -> S {
    use futures::future::Either;

    let mut transition = None;
    while transition.is_none() {
        let timer = tokio_timer::Timer::default();
        let timeout = timer.sleep(Duration::from_secs(3));
        let (event, receiver2) = match receiver.into_future().select2(timeout).wait() {
            Ok(Either::A((stream_result, _timer))) => stream_result,
            _ => panic!("Timed out waiting for tunnel state transition"),
        };
        receiver = receiver2;
        if let DaemonEvent::TunnelState(new_state) = event.unwrap() {
            transition = Some(new_state);
        }
    }

    assert_eq!(transition.unwrap(), expected_state);
    receiver
}

fn create_mock_openvpn_plugin_client<P: AsRef<Path>>(
    openvpn_args_file_path: P,
) -> MockOpenVpnPluginRpcClient {
    let address = get_plugin_arguments(openvpn_args_file_path);

    MockOpenVpnPluginRpcClient::new(address)
        .expect("Failed to create mock RPC client to connect to OpenVPN plugin event listener")
}

fn get_plugin_arguments<P: AsRef<Path>>(openvpn_args_file_path: P) -> String {
    let mut arguments = search_openvpn_args(openvpn_args_file_path, OPENVPN_PLUGIN_NAME).skip(1);

    arguments
        .next()
        .expect("Missing OpenVPN plugin RPC listener address argument")
        .expect("Failed to read from mock OpenVPN arguments file")
}
