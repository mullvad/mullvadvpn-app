package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.compose.dialog.payment.PaymentDialogData
import net.mullvad.mullvadvpn.model.TunnelState

data class OutOfTimeUiState(
    val tunnelState: TunnelState = TunnelState.Disconnected(),
    val deviceName: String = "",
    val billingPaymentState: PaymentState? = null,
    val paymentDialogData: PaymentDialogData? = null
)
