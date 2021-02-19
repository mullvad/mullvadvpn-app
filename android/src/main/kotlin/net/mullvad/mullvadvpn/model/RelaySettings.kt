package net.mullvad.mullvadvpn.model

sealed class RelaySettings {
    object CustomTunnelEndpoint : RelaySettings()
    class Normal(var relayConstraints: RelayConstraints) : RelaySettings()
}
