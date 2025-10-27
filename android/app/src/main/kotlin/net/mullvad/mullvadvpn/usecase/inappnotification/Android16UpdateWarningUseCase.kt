package net.mullvad.mullvadvpn.usecase.inappnotification

import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.transformLatest
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.lib.model.InAppNotification
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.repository.UserPreferencesRepository

class Android16UpdateWarningUseCase(
    private val userPreferencesRepository: UserPreferencesRepository,
    private val managementService: ManagementService,
) : InAppNotificationUseCase {
    @OptIn(ExperimentalCoroutinesApi::class)
    override operator fun invoke(): Flow<InAppNotification?> =
        combine(
                userPreferencesRepository
                    .preferencesFlow()
                    .map { it.showAndroid16ConnectWarning }
                    .distinctUntilChanged(),
                managementService.tunnelState.map { it.toTunState() }.distinctUntilChanged(),
            ) { showWarning, tunState ->
                showWarning to tunState
            }
            .transformLatest { (showWarning, tunState) ->
                when {
                    showWarning && tunState == SimpleTunState.Connecting -> {
                        emit(null)
                        delay(SHOW_WARNING_DELAY)
                        emit(InAppNotification.Android16UpgradeWarning)
                    }
                    showWarning && tunState == SimpleTunState.Connected -> {
                        // User is connected, we know this warning is not relevant so we remove it
                        // and don't show the warning again.
                        userPreferencesRepository.setShowAndroid16ConnectWarning(false)
                        emit(null)
                    }
                    else -> emit(null)
                }
            }

    private fun TunnelState.toTunState(): SimpleTunState =
        when (this) {
            is TunnelState.Connecting -> SimpleTunState.Connecting
            is TunnelState.Disconnecting if
                actionAfterDisconnect == ActionAfterDisconnect.Reconnect
             -> SimpleTunState.Connecting
            is TunnelState.Connected -> SimpleTunState.Connected
            else -> SimpleTunState.Other
        }

    private enum class SimpleTunState {
        Connecting,
        Connected,
        Other,
    }

    companion object {
        private val SHOW_WARNING_DELAY = 2.seconds
    }
}
