package net.mullvad.mullvadvpn.feature.home.impl.outoftime

import net.mullvad.mullvadvpn.lib.model.TunnelState

data class OutOfTimeUiState(
    val tunnelState: TunnelState = TunnelState.Disconnected(),
    val deviceName: String = "",
    val showSitePayment: Boolean = false,
    val verificationPending: Boolean = false,
)
