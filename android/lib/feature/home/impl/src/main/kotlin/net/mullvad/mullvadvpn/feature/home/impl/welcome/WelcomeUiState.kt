package net.mullvad.mullvadvpn.feature.home.impl.welcome

import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus

data class WelcomeUiState(
    val tunnelState: TunnelState,
    val accountNumber: AccountNumber?,
    val deviceName: String?,
    val showSitePayment: Boolean,
    val paymentStatus: PaymentStatus?,
)
