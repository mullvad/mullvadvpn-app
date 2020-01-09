package net.mullvad.mullvadvpn.model

sealed class RelaySettingsUpdate {
    class CustomTunnelEndpoint() : RelaySettingsUpdate()
    class RelayConstraintsUpdate(var location: Constraint<LocationConstraint>?) :
        RelaySettingsUpdate()

    class Normal(var constraints: RelayConstraintsUpdate) : RelaySettingsUpdate() {
        fun get0() = constraints
    }
}
