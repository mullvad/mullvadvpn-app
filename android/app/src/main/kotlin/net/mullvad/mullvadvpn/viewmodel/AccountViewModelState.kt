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
                AccountUiState.DefaultUiState(
                    deviceName = deviceState.deviceName() ?: "",
                    accountNumber = deviceState.token() ?: "",
                    accountExpiry = accountExpiry.date()
                )
            is AccountScreenDialogState.DeviceNameInfoDialog ->
                AccountUiState.DeviceNameDialogUiState(
                    deviceName = deviceState.deviceName() ?: "",
                    accountNumber = deviceState.token() ?: "",
                    accountExpiry = accountExpiry.date()
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
