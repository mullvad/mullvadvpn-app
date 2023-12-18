package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.ConnectUiState
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.repository.InAppNotificationController
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.usecase.NewDeviceNotificationUseCase
import net.mullvad.mullvadvpn.usecase.OutOfTimeUseCase
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.usecase.RelayListUseCase
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier
import net.mullvad.mullvadvpn.util.combine
import net.mullvad.mullvadvpn.util.daysFromNow
import net.mullvad.mullvadvpn.util.toInAddress
import net.mullvad.mullvadvpn.util.toOutAddress
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

@OptIn(FlowPreview::class)
class ConnectViewModel(
    private val serviceConnectionManager: ServiceConnectionManager,
    accountRepository: AccountRepository,
    private val deviceRepository: DeviceRepository,
    private val inAppNotificationController: InAppNotificationController,
    private val newDeviceNotificationUseCase: NewDeviceNotificationUseCase,
    private val relayListUseCase: RelayListUseCase,
    private val outOfTimeUseCase: OutOfTimeUseCase,
    private val paymentUseCase: PaymentUseCase
) : ViewModel() {
    private val _uiSideEffect = Channel<UiSideEffect>(1, BufferOverflow.DROP_OLDEST)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val _shared: SharedFlow<ServiceConnectionContainer> =
        serviceConnectionManager.connectionState
            .flatMapLatest { state ->
                if (state is ServiceConnectionState.ConnectedReady) {
                    flowOf(state.container)
                } else {
                    emptyFlow()
                }
            }
            .shareIn(viewModelScope, SharingStarted.WhileSubscribed())

    val uiState: StateFlow<ConnectUiState> =
        _shared
            .flatMapLatest { serviceConnection ->
                combine(
                    relayListUseCase.selectedRelayItem(),
                    inAppNotificationController.notifications,
                    serviceConnection.connectionProxy.tunnelUiStateFlow(),
                    serviceConnection.connectionProxy.tunnelRealStateFlow(),
                    serviceConnection.connectionProxy.lastKnownDisconnectedLocation(),
                    accountRepository.accountExpiryState,
                    deviceRepository.deviceState.map { it.deviceName() }
                ) {
                    relayLocation,
                    notifications,
                    tunnelUiState,
                    tunnelRealState,
                    lastKnownDisconnectedLocation,
                    accountExpiry,
                    deviceName ->
                    ConnectUiState(
                        location =
                            when (tunnelRealState) {
                                is TunnelState.Disconnected -> tunnelRealState.location()
                                        ?: lastKnownDisconnectedLocation
                                is TunnelState.Connecting -> tunnelRealState.location
                                        ?: relayLocation?.location?.location
                                is TunnelState.Connected -> tunnelRealState.location
                                is TunnelState.Disconnecting -> lastKnownDisconnectedLocation
                                is TunnelState.Error -> null
                            },
                        relayLocation = relayLocation,
                        tunnelUiState = tunnelUiState,
                        tunnelRealState = tunnelRealState,
                        inAddress =
                            when (tunnelRealState) {
                                is TunnelState.Connected -> tunnelRealState.endpoint.toInAddress()
                                is TunnelState.Connecting -> tunnelRealState.endpoint?.toInAddress()
                                else -> null
                            },
                        outAddress = tunnelRealState.location()?.toOutAddress() ?: "",
                        showLocation =
                            when (tunnelUiState) {
                                is TunnelState.Disconnected -> true
                                is TunnelState.Disconnecting -> {
                                    when (tunnelUiState.actionAfterDisconnect) {
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
                        daysLeftUntilExpiry = accountExpiry.date()?.daysFromNow()
                    )
                }
            }
            .debounce(UI_STATE_DEBOUNCE_DURATION_MILLIS)
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), ConnectUiState.INITIAL)

    init {
        viewModelScope.launch {
            // This once we get isOutOfTime true we will navigate to OutOfTime view.
            outOfTimeUseCase.isOutOfTime().first { it == true }
            _uiSideEffect.send(UiSideEffect.OutOfTime)
        }

        viewModelScope.launch {
            paymentUseCase.verifyPurchases { accountRepository.fetchAccountExpiry() }
        }
    }

    private fun ConnectionProxy.tunnelUiStateFlow(): Flow<TunnelState> =
        callbackFlowFromNotifier(this.onUiStateChange)

    private fun ConnectionProxy.tunnelRealStateFlow(): Flow<TunnelState> =
        callbackFlowFromNotifier(this.onStateChange)

    private fun ConnectionProxy.lastKnownDisconnectedLocation(): Flow<GeoIpLocation?> =
        tunnelRealStateFlow()
            .filterIsInstance<TunnelState.Disconnected>()
            .filter { it.location != null }
            .map { it.location }
            .onStart { emit(null) }

    fun onDisconnectClick() {
        serviceConnectionManager.connectionProxy()?.disconnect()
    }

    fun onReconnectClick() {
        serviceConnectionManager.connectionProxy()?.reconnect()
    }

    fun onConnectClick() {
        serviceConnectionManager.connectionProxy()?.connect()
    }

    fun onCancelClick() {
        serviceConnectionManager.connectionProxy()?.disconnect()
    }

    fun onManageAccountClick() {
        viewModelScope.launch {
            _uiSideEffect.send(
                UiSideEffect.OpenAccountManagementPageInBrowser(
                    serviceConnectionManager.authTokenCache()?.fetchAuthToken() ?: ""
                )
            )
        }
    }

    fun dismissNewDeviceNotification() {
        newDeviceNotificationUseCase.clearNewDeviceCreatedNotification()
    }

    sealed interface UiSideEffect {
        data class OpenAccountManagementPageInBrowser(val token: String) : UiSideEffect

        data object OutOfTime : UiSideEffect
    }

    companion object {
        const val UI_STATE_DEBOUNCE_DURATION_MILLIS: Long = 200
    }
}
