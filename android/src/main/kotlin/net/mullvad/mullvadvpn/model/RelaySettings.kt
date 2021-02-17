package net.mullvad.mullvadvpn.model

sealed class RelaySettings {
    class CustomTunnelEndpoint() : RelaySettings() {
        companion object {
            @JvmStatic
            val INSTANCE = CustomTunnelEndpoint()
        }
    }

    class Normal(var relayConstraints: RelayConstraints) : RelaySettings()
}
