package net.mullvad.mullvadvpn.model

sealed class RelaySettings {
    class CustomTunnelEndpoint() : RelaySettings()
    class Normal(var relayConstraints: RelayConstraints) : RelaySettings()
}
