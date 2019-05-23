package net.mullvad.mullvadvpn.model

sealed class RelaySettings {
    class CustomTunnelEndpoint() : RelaySettings()
    class RelayConstraints(var location: Constraint<LocationConstraint>) : RelaySettings()
}
