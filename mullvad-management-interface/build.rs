fn main() {
    const PROTO_FILE: &str = "proto/management_interface.proto";
    const PROTO_DIR: &str = "proto";
    tonic_build::configure()
        .type_attribute(
            "mullvad_daemon.management_interface.RelayListCountry",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "mullvad_daemon.management_interface.RelayListCity",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "mullvad_daemon.management_interface.Relay",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "mullvad_daemon.management_interface.RelayTunnels",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "mullvad_daemon.management_interface.RelayBridges",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "mullvad_daemon.management_interface.Location",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "mullvad_daemon.management_interface.OpenVpnEndpointData",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "mullvad_daemon.management_interface.WireguardEndpointData",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "mullvad_daemon.management_interface.ShadowsocksEndpointData",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "mullvad_daemon.management_interface.PortRange",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .compile(&[PROTO_FILE], &[PROTO_DIR])
        .unwrap();
    println!("cargo:rerun-if-changed={}", PROTO_FILE);
}
