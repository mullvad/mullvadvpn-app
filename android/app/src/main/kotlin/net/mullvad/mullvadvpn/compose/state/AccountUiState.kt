package net.mullvad.mullvadvpn.compose.state

import org.joda.time.DateTime

data class AccountUiState(
    val deviceName: String,
    val accountNumber: String,
    val accountExpiry: DateTime?,
    val showDeviceInfoDialog: Boolean = false
) {
    companion object {
        fun defaultInstance(): AccountUiState {
            return AccountUiState("", "", null, false)
        }
    }
}
