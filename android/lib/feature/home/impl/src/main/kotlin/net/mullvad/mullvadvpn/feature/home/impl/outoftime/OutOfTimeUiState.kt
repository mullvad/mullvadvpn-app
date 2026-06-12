package net.mullvad.mullvadvpn.feature.home.impl.outoftime

import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus

data class OutOfTimeUiState(
    val tunnelState: TunnelState = TunnelState.Disconnected(),
    val deviceName: String? = null,
    val showSitePayment: Boolean = false,
    val paymentStatus: PaymentStatus? = null,
) {
    init {
        require(deviceName?.isBlank() != true) { "deviceName cannot be blank or empty" }
    }
}
