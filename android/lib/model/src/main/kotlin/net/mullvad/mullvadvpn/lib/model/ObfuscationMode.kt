package net.mullvad.mullvadvpn.lib.model

enum class ObfuscationMode {
    Auto,
    Off,
    Udp2Tcp,
    Shadowsocks,
    Quic,
    Lwo,
    WireguardPort,
}
