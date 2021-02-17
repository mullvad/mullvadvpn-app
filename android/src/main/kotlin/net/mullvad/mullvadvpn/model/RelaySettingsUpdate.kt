package net.mullvad.mullvadvpn.model

sealed class RelaySettingsUpdate {
    class CustomTunnelEndpoint() : RelaySettingsUpdate() {
        companion object {
            @JvmStatic
            val INSTANCE = CustomTunnelEndpoint()
        }
    }

    data class Normal(var constraints: RelayConstraintsUpdate) : RelaySettingsUpdate() {
        fun get0() = constraints
    }
}
