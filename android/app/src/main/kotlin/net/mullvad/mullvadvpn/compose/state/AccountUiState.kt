package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.DeviceState
import org.joda.time.DateTime

data class AccountUiState(
    val deviceName: String?,
    val accountNumber: String?,
    val accountExpiry: DateTime?,
    val billingPaymentState: PaymentState = PaymentState.Loading,
    val dialogState: AccountScreenDialogState = AccountScreenDialogState.NoDialog
) {
    companion object {
        fun default() =
            AccountUiState(
                deviceName = DeviceState.Unknown.deviceName(),
                accountNumber = DeviceState.Unknown.token(),
                accountExpiry = AccountExpiry.Missing.date(),
                dialogState = AccountScreenDialogState.NoDialog
            )
    }
}

sealed interface AccountScreenDialogState {
    data object NoDialog : AccountScreenDialogState

    //Billing dialogs
    data object VerificationError: AccountScreenDialogState

    data object PurchaseError: AccountScreenDialogState

    data object PurchaseComplete: AccountScreenDialogState
}
