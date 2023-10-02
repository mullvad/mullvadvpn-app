package net.mullvad.mullvadvpn.model

sealed class RelaySettingsUpdate {
    object CustomTunnelEndpoint : RelaySettingsUpdate()

    data class Normal(val constraints: RelayConstraintsUpdate) : RelaySettingsUpdate()
}
