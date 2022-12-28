package net.mullvad.mullvadvpn.test.e2e.misc

data class ConnCheckState(
    val isConnected: Boolean,
    val ipAddress: String
)
