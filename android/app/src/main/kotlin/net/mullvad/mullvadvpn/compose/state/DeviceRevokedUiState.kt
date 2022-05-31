package net.mullvad.mullvadvpn.compose.state

data class DeviceRevokedUiState(
    val isSecured: Boolean
) {
    companion object {
        val DEFAULT = DeviceRevokedUiState(
            isSecured = false
        )
    }
}
