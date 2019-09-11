package net.mullvad.mullvadvpn.model

data class Relay(val hostname: String, val hasWireguardTunnels: Boolean, val active: Boolean) {
}
