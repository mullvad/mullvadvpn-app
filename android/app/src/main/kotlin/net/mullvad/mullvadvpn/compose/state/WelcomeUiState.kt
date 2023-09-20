package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.model.TunnelState

data class WelcomeUiState(
    val tunnelState: TunnelState = TunnelState.Disconnected,
    val accountNumber: String? = null,
    val deviceName: String? = null,
    val billingPaymentState: PaymentState = PaymentState.Loading,
    val purchaseResult: PurchaseResult? = null
)
