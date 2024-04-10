package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.AccountToken
import net.mullvad.mullvadvpn.model.TunnelState

data class WelcomeUiState(
    val tunnelState: TunnelState = TunnelState.Disconnected(),
    val accountNumber: AccountToken? = null,
    val deviceName: String? = null,
    val showSitePayment: Boolean = false,
    val billingPaymentState: PaymentState? = null,
)
