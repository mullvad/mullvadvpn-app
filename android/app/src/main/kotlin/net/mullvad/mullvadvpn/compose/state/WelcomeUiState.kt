package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.TunnelState

data class WelcomeUiState(
    val tunnelState: TunnelState,
    val accountNumber: AccountNumber?,
    val deviceName: String?,
    val showSitePayment: Boolean,
)
