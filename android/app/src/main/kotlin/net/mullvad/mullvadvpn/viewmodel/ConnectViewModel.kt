package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.compose.state.ConnectUiState
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.LocationInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.util.appVersionCallbackFlow
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier
import net.mullvad.mullvadvpn.util.combine
import net.mullvad.mullvadvpn.util.toInAddress
import net.mullvad.mullvadvpn.util.toOutAddress
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

class ConnectViewModel(private val serviceConnectionManager: ServiceConnectionManager) :
    ViewModel() {
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
                    serviceConnection.relayListListener.relayListCallbackFlow(),
                    serviceConnection.appVersionInfoCache.appVersionCallbackFlow(),
                    serviceConnection.connectionProxy.tunnelUiStateFlow(),
                    serviceConnection.connectionProxy.tunnelRealStateFlow(),
                    _isTunnelInfoExpanded
                ) {
                    location,
                    relayLocation,
                    versionInfo,
                    tunnelUiState,
                    tunnelRealState,
                    isTunnelInfoExpanded ->
                    ConnectUiState(
                        location =
                            when (tunnelRealState) {
                                is TunnelState.Connected -> tunnelRealState.location
                                is TunnelState.Connecting -> tunnelRealState.location
                                else -> null
                            }
                                ?: location,
                        relayLocation = relayLocation,
                        versionInfo = versionInfo,
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
                            }
                    )
                }
            }
            .debounce(UI_STATE_DEBOUNCE_DURATION_MILLIS)
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), ConnectUiState.INITIAL)

    private fun LocationInfoCache.locationCallbackFlow() = callbackFlow {
        onNewLocation = { this.trySend(it) }
        awaitClose { onNewLocation = null }
    }

    private fun RelayListListener.relayListCallbackFlow() = callbackFlow {
        onRelayCountriesChange = { _, item -> this.trySend(item) }
        awaitClose { onRelayCountriesChange = null }
    }

    private fun ConnectionProxy.tunnelUiStateFlow(): Flow<TunnelState> =
        callbackFlowFromNotifier(this.onUiStateChange)

    private fun ConnectionProxy.tunnelRealStateFlow(): Flow<TunnelState> =
        callbackFlowFromNotifier(this.onStateChange)

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

    companion object {
        const val UI_STATE_DEBOUNCE_DURATION_MILLIS: Long = 200
    }
}
