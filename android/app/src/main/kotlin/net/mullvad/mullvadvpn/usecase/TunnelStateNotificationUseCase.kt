package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier
import net.mullvad.mullvadvpn.util.flatMapReadyConnectionOrDefault
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

class TunnelStateNotificationUseCase(
    private val serviceConnectionManager: ServiceConnectionManager,
) {
    fun notifications(): Flow<List<InAppNotification>> =
        serviceConnectionManager.connectionState
            .flatMapReadyConnectionOrDefault(flowOf(emptyList())) {
                it.container.connectionProxy
                    .tunnelUiStateFlow()
                    .distinctUntilChanged()
                    .map(::tunnelStateNotification)
                    .map(::listOfNotNull)
            }
            .distinctUntilChanged()

    private fun tunnelStateNotification(tunnelUiState: TunnelState): InAppNotification? =
        when (tunnelUiState) {
            is TunnelState.Connecting -> InAppNotification.TunnelStateBlocked
            is TunnelState.Disconnecting -> {
                if (
                    tunnelUiState.actionAfterDisconnect == ActionAfterDisconnect.Block ||
                        tunnelUiState.actionAfterDisconnect == ActionAfterDisconnect.Reconnect
                ) {
                    InAppNotification.TunnelStateBlocked
                } else null
            }
            is TunnelState.Error -> InAppNotification.TunnelStateError(tunnelUiState.errorState)
            is TunnelState.Connected,
            is TunnelState.Disconnected -> null
        }

    private fun ConnectionProxy.tunnelUiStateFlow(): Flow<TunnelState> =
        callbackFlowFromNotifier(this.onUiStateChange)
}
