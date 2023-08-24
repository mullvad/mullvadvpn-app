package net.mullvad.mullvadvpn.viewmodel

import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.DeviceState
import org.joda.time.DateTime

data class AccountViewModelState(
    val deviceName: String?,
    val accountNumber: String?,
    val accountExpiry: DateTime?,
    val dialogState: AccountScreenDialogState = AccountScreenDialogState.NoDialog
) {
    companion object {
        fun default() =
            AccountViewModelState(
                deviceName = DeviceState.Unknown.deviceName(),
                accountNumber = DeviceState.Unknown.token(),
                accountExpiry = AccountExpiry.Missing.date(),
                dialogState = AccountScreenDialogState.NoDialog
            )
    }
}

sealed class AccountScreenDialogState {
    data object NoDialog : AccountScreenDialogState()

    data object DeviceNameInfoDialog : AccountScreenDialogState()
}
