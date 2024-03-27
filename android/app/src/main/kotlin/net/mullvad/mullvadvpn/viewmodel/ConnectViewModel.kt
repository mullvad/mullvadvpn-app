package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.ConnectUiState
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.repository.InAppNotificationController
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.usecase.NewDeviceNotificationUseCase
import net.mullvad.mullvadvpn.usecase.OutOfTimeUseCase
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.usecase.RelayListUseCase
import net.mullvad.mullvadvpn.util.combine
import net.mullvad.mullvadvpn.util.daysFromNow
import net.mullvad.mullvadvpn.util.toInAddress
import net.mullvad.mullvadvpn.util.toOutAddress
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

@OptIn(FlowPreview::class)
class ConnectViewModel(
    accountRepository: AccountRepository,
    private val deviceRepository: DeviceRepository,
    private val inAppNotificationController: InAppNotificationController,
    private val newDeviceNotificationUseCase: NewDeviceNotificationUseCase,
    private val relayListUseCase: RelayListUseCase,
    private val outOfTimeUseCase: OutOfTimeUseCase,
    private val paymentUseCase: PaymentUseCase,
    private val connectionProxy: ConnectionProxy,
    private val isPlayBuild: Boolean
) : ViewModel() {
    private val _uiSideEffect = Channel<UiSideEffect>()

    val uiSideEffect =
        merge(_uiSideEffect.receiveAsFlow(), outOfTimeEffect(), revokedDeviceEffect())

    val uiState: StateFlow<ConnectUiState> =
        combine(
                relayListUseCase.selectedRelayItem(),
                inAppNotificationController.notifications,
                connectionProxy.tunnelState,
                connectionProxy.lastKnownDisconnectedLocation(),
                accountRepository.accountExpiry,
                deviceRepository.deviceState.map { it.deviceName() }
            ) {
                selectedRelayItem,
                notifications,
                tunnelState,
                lastKnownDisconnectedLocation,
                accountExpiry,
                deviceName ->
                ConnectUiState(
                    location =
                        when (tunnelState) {
                            is TunnelState.Disconnected ->
                                tunnelState.location() ?: lastKnownDisconnectedLocation
                            is TunnelState.Connecting ->
                                tunnelState.location ?: selectedRelayItem?.location()
                            is TunnelState.Connected -> tunnelState.location
                            is TunnelState.Disconnecting -> lastKnownDisconnectedLocation
                            is TunnelState.Error -> null
                        },
                    selectedRelayItem = selectedRelayItem,
                    tunnelState = tunnelState,
                    inAddress =
                        when (tunnelState) {
                            is TunnelState.Connected -> tunnelState.endpoint.toInAddress()
                            is TunnelState.Connecting -> tunnelState.endpoint?.toInAddress()
                            else -> null
                        },
                    outAddress = tunnelState.location()?.toOutAddress() ?: "",
                    showLocation =
                        when (tunnelState) {
                            is TunnelState.Disconnected -> true
                            is TunnelState.Disconnecting -> {
                                when (tunnelState.actionAfterDisconnect) {
                                    ActionAfterDisconnect.Nothing -> false
                                    ActionAfterDisconnect.Block -> true
                                    ActionAfterDisconnect.Reconnect -> false
                                }
                            }
                            is TunnelState.Connecting -> false
                            is TunnelState.Connected -> false
                            is TunnelState.Error -> true
                        },
                    inAppNotification = notifications.firstOrNull(),
                    deviceName = deviceName,
                    daysLeftUntilExpiry = accountExpiry.date()?.daysFromNow(),
                    isPlayBuild = isPlayBuild,
                )
            }
            .debounce(UI_STATE_DEBOUNCE_DURATION_MILLIS)
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), ConnectUiState.INITIAL)

    init {
        viewModelScope.launch {
            paymentUseCase.verifyPurchases {
                viewModelScope.launch { accountRepository.getAccountExpiry() }
            }
        }
    }

    private fun ConnectionProxy.lastKnownDisconnectedLocation(): Flow<GeoIpLocation?> =
        tunnelState
            .filterIsInstance<TunnelState.Disconnected>()
            .filter { it.location != null }
            .map { it.location }
            .onStart { emit(null) }

    fun onDisconnectClick() {
        viewModelScope.launch { connectionProxy.disconnect() }
    }

    fun onReconnectClick() {
        viewModelScope.launch { connectionProxy.reconnect() }
    }

    fun onConnectClick() {
        viewModelScope.launch { connectionProxy.connect() }
    }

    fun onCancelClick() {
        viewModelScope.launch { connectionProxy.disconnect() }
    }

    fun onManageAccountClick() {
        viewModelScope.launch {
            _uiSideEffect.trySend(
                UiSideEffect.OpenAccountManagementPageInBrowser(
                    TODO() // serviceConnectionManager.authTokenCache()?.fetchAuthToken() ?: ""
                )
            )
        }
    }

    fun dismissNewDeviceNotification() {
        newDeviceNotificationUseCase.clearNewDeviceCreatedNotification()
    }

    private fun outOfTimeEffect() =
        outOfTimeUseCase.isOutOfTime.filter { it == true }.map { UiSideEffect.OutOfTime }

    private fun revokedDeviceEffect() =
        deviceRepository.deviceState.filterIsInstance<DeviceState.Revoked>().map {
            UiSideEffect.RevokedDevice
        }

    sealed interface UiSideEffect {
        data class OpenAccountManagementPageInBrowser(val token: String) : UiSideEffect

        data object OutOfTime : UiSideEffect

        data object RevokedDevice : UiSideEffect
    }

    companion object {
        const val UI_STATE_DEBOUNCE_DURATION_MILLIS: Long = 200
    }
}
