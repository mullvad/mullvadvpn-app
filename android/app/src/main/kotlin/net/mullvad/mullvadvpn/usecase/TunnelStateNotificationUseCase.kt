package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.InAppNotification
import net.mullvad.mullvadvpn.lib.model.ParameterGenerationError
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.util.inAnyOf

class TunnelStateNotificationUseCase(
    private val connectionProxy: ConnectionProxy,
    private val relayListRepository: RelayListRepository,
    private val settingsRepository: SettingsRepository,
) {
    @OptIn(ExperimentalCoroutinesApi::class)
    operator fun invoke(): Flow<List<InAppNotification>> =
        connectionProxy.tunnelState
            .distinctUntilChanged()
            .map(::tunnelStateNotification)
            .flatMapLatest { inAppNotification ->
                combine(relayListRepository.portRanges, settingsRepository.settingsUpdates) {
                    portRanges,
                    settings ->
                    inAppNotification?.maybeUpdateWithPortError(
                        wireguardPort = settings.wireguardPort(),
                        availablePorts = portRanges,
                    )
                }
            }
            .map(::listOfNotNull)
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

    private fun InAppNotification.maybeUpdateWithPortError(
        wireguardPort: Constraint<Port>,
        availablePorts: List<PortRange>,
    ): InAppNotification =
        if (this is InAppNotification.TunnelStateError && error.isPossiblePortError()) {
            wireguardPort.invalidPortOrNull(availablePorts)?.let {
                copy(
                    error =
                        ErrorState(
                            cause = ErrorStateCause.NoRelaysMatchSelectedPort(port = it),
                            isBlocking = error.isBlocking,
                        )
                )
            } ?: this
        } else this

    private fun ErrorState.isPossiblePortError(): Boolean =
        cause is ErrorStateCause.TunnelParameterError &&
            (cause as ErrorStateCause.TunnelParameterError).error ==
                ParameterGenerationError.NoMatchingRelay

    private fun Constraint<Port>.invalidPortOrNull(availablePortRanges: List<PortRange>): Port? =
        getOrNull()?.takeIf { !it.inAnyOf(availablePortRanges) }

    private fun Settings?.wireguardPort() =
        this?.relaySettings?.relayConstraints?.wireguardConstraints?.port ?: Constraint.Any
}
