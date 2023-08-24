package net.mullvad.mullvadvpn.compose.state

import org.joda.time.DateTime

sealed interface AccountUiState {
    val deviceName: String
    val accountNumber: String
    val accountExpiry: DateTime?

    data class DefaultUiState(
        override val deviceName: String = "",
        override val accountNumber: String = "",
        override val accountExpiry: DateTime? = null
    ) : AccountUiState

    data class DeviceNameDialogUiState(
        override val deviceName: String,
        override val accountNumber: String,
        override val accountExpiry: DateTime?
    ) : AccountUiState
}
