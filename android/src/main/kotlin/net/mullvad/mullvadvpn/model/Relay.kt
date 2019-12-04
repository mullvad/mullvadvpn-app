package net.mullvad.mullvadvpn.model

data class Relay(val hostname: String, val active: Boolean, val tunnels: RelayTunnels) {
    val hasWireguardTunnels
        get() = !tunnels.wireguard.isEmpty()
}
