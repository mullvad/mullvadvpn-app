package net.mullvad.mullvadvpn.feature.home.impl.welcome

import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.TunnelState

data class WelcomeUiState(
    val tunnelState: TunnelState,
    val accountNumber: AccountNumber?,
    val deviceName: String?,
    val showSitePayment: Boolean,
    val verificationPending: Boolean,
)
