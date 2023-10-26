package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.ConnectUiState
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.repository.InAppNotificationController
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.LocationInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.usecase.NewDeviceNotificationUseCase
import net.mullvad.mullvadvpn.usecase.RelayListUseCase
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier
import net.mullvad.mullvadvpn.util.combine
import net.mullvad.mullvadvpn.util.daysFromNow
import net.mullvad.mullvadvpn.util.toInAddress
import net.mullvad.mullvadvpn.util.toOutAddress
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.tunnel.ErrorStateCause

@OptIn(FlowPreview::class)
class ConnectViewModel(
    private val serviceConnectionManager: ServiceConnectionManager,
    accountRepository: AccountRepository,
    private val deviceRepository: DeviceRepository,
    private val inAppNotificationController: InAppNotificationController,
    private val newDeviceNotificationUseCase: NewDeviceNotificationUseCase,
    private val relayListUseCase: RelayListUseCase
) : ViewModel() {
    private val _uiSideEffect = MutableSharedFlow<UiSideEffect>(extraBufferCapacity = 1)
    val uiSideEffect = _uiSideEffect.asSharedFlow()

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

    private val _isTunnelInfoExpanded = MutableStateFlow(false)

    val uiState: StateFlow<ConnectUiState> =
        _shared
            .flatMapLatest { serviceConnection ->
                combine(
                    serviceConnection.locationInfoCache.locationCallbackFlow(),
                    relayListUseCase.relayListWithSelection(),
                    inAppNotificationController.notifications,
                    serviceConnection.connectionProxy.tunnelUiStateFlow(),
                    serviceConnection.connectionProxy.tunnelRealStateFlow(),
                    accountRepository.accountExpiryState,
                    _isTunnelInfoExpanded,
                    deviceRepository.deviceState.map { it.deviceName() }
                ) {
                    location,
                    relayLocation,
                    notifications,
                    tunnelUiState,
                    tunnelRealState,
                    accountExpiry,
                    isTunnelInfoExpanded,
                    deviceName ->
                    if (tunnelRealState.isTunnelErrorStateDueToExpiredAccount()) {
                        _uiSideEffect.tryEmit(UiSideEffect.OpenOutOfTimeView)
                    }
                    ConnectUiState(
                        location =
                            when (tunnelRealState) {
                                is TunnelState.Connected -> tunnelRealState.location
                                is TunnelState.Connecting -> tunnelRealState.location
                                else -> null
                            }
                                ?: location,
                        relayLocation = relayLocation.second,
                        tunnelUiState = tunnelUiState,
                        tunnelRealState = tunnelRealState,
                        isTunnelInfoExpanded = isTunnelInfoExpanded,
                        inAddress =
                            when (tunnelRealState) {
                                is TunnelState.Connected -> tunnelRealState.endpoint.toInAddress()
                                is TunnelState.Connecting -> tunnelRealState.endpoint?.toInAddress()
                                else -> null
                            },
                        outAddress = location?.toOutAddress() ?: "",
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

    private fun LocationInfoCache.locationCallbackFlow() = callbackFlow {
        onNewLocation = { this.trySend(it) }
        awaitClose { onNewLocation = null }
    }

    private fun ConnectionProxy.tunnelUiStateFlow(): Flow<TunnelState> =
        callbackFlowFromNotifier(this.onUiStateChange)

    private fun ConnectionProxy.tunnelRealStateFlow(): Flow<TunnelState> =
        callbackFlowFromNotifier(this.onStateChange)

    private fun TunnelState.isTunnelErrorStateDueToExpiredAccount(): Boolean {
        return ((this as? TunnelState.Error)?.errorState?.cause as? ErrorStateCause.AuthFailed)
            ?.isCausedByExpiredAccount()
            ?: false
    }

    fun toggleTunnelInfoExpansion() {
        _isTunnelInfoExpanded.value = _isTunnelInfoExpanded.value.not()
    }

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
            _uiSideEffect.tryEmit(
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

        data object OpenOutOfTimeView : UiSideEffect
    }

    companion object {
        const val UI_STATE_DEBOUNCE_DURATION_MILLIS: Long = 200
    }
}
