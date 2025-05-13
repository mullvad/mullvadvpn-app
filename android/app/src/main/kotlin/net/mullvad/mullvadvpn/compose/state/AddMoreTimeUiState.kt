package net.mullvad.mullvadvpn.compose.state

data class AddMoreTimeUiState(
    val billingPaymentState: PaymentState?,
    val showSitePayment: Boolean,
    val showManageAccountLoading: Boolean,
)
