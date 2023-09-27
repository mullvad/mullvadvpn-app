package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.TunnelState

data class OutOfTimeUiState(
    val tunnelState: TunnelState = TunnelState.Disconnected,
    val billingPaymentState: PaymentState = PaymentState.Loading,
    val dialogState: OutOfTimeDialogState = OutOfTimeDialogState.NoDialog
)
