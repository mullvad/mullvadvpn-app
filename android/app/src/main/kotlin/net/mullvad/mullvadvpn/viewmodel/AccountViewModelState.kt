package net.mullvad.mullvadvpn.viewmodel

import net.mullvad.mullvadvpn.compose.state.AccountUiState
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.DeviceState

data class AccountViewModelState(
    val deviceState: DeviceState,
    val accountExpiry: AccountExpiry,
    val dialogState: AccountScreenDialogState
) {
    fun toUiState(): AccountUiState {
        return when (dialogState) {
            is AccountScreenDialogState.NoDialog ->
                AccountUiState(
                    deviceName = deviceState.deviceName() ?: "",
                    accountNumber = deviceState.token() ?: "",
                    accountExpiry = accountExpiry.date(),
                    showDeviceInfoDialog = false
                )
            is AccountScreenDialogState.DeviceNameInfoDialog ->
                AccountUiState(
                    deviceName = deviceState.deviceName() ?: "",
                    accountNumber = deviceState.token() ?: "",
                    accountExpiry = accountExpiry.date(),
                    showDeviceInfoDialog = true
                )
        }
    }

    companion object {
        fun default() =
            AccountViewModelState(
                deviceState = DeviceState.Unknown,
                accountExpiry = AccountExpiry.Missing,
                dialogState = AccountScreenDialogState.NoDialog
            )
    }
}

sealed class AccountScreenDialogState {
    data object NoDialog : AccountScreenDialogState()

    data object DeviceNameInfoDialog : AccountScreenDialogState()
}
