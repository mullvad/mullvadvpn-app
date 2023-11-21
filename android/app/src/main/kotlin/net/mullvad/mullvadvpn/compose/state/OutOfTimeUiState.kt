package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.TunnelState

data class OutOfTimeUiState(
    val tunnelState: TunnelState = TunnelState.Disconnected,
    val deviceName: String = "",
    val showSitePayment: Boolean = false,
    val billingPaymentState: PaymentState? = null,
)
