package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.TunnelState

data class OutOfTimeUiState(
    val tunnelState: TunnelState = TunnelState.Disconnected(),
    val deviceName: String = "",
    val showSitePayment: Boolean = false,
    val verificationPending: Boolean = false,
)
