package net.mullvad.mullvadvpn.compose.state

import org.joda.time.DateTime

data class AccountUiState(
    val deviceName: String = "",
    val accountNumber: String = "",
    val accountExpiry: DateTime? = null,
    val webPaymentAvailable: Boolean = false,
    val billingPaymentState: PaymentState = PaymentState.Loading,
    val purchaseLoading: Boolean = false,
    val dialogState: AccountDialogState = AccountDialogState.NoDialog
)
