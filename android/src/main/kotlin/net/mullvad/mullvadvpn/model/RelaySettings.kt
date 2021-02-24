package net.mullvad.mullvadvpn.model

sealed class RelaySettings {
    object CustomTunnelEndpoint : RelaySettings()
    class Normal(val relayConstraints: RelayConstraints) : RelaySettings()
}
