package net.mullvad.mullvadvpn.model

sealed class RelaySettingsUpdate {
    class CustomTunnelEndpoint() : RelaySettingsUpdate()

    data class Normal(var constraints: RelayConstraintsUpdate) : RelaySettingsUpdate() {
        fun get0() = constraints
    }
}
