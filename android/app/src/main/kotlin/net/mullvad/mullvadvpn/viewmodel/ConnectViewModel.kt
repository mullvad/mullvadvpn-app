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
import net.mullvad.mullvadvpn.util.appVersionCallbackFlow
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier
import net.mullvad.mullvadvpn.util.combine

class ConnectViewModel(serviceConnectionManager: ServiceConnectionManager) : ViewModel() {
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
                        location = location,
                        relayLocation = relayLocation,
                        versionInfo = versionInfo,
                        tunnelUiState = tunnelUiState,
                        tunnelRealState = tunnelRealState,
                        isTunnelInfoExpanded = isTunnelInfoExpanded
                    )
                }
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), ConnectUiState.INITIAL)

    private fun LocationInfoCache.locationCallbackFlow() = callbackFlow {
        onNewLocation = { this.trySend(it) }
        awaitClose { onNewLocation = null }
    }

    private fun RelayListListener.relayListCallbackFlow() = callbackFlow {
        onRelayListChange = { _, item -> this.trySend(item) }
        awaitClose { onRelayListChange = null }
    }

    private fun ConnectionProxy.tunnelUiStateFlow(): Flow<TunnelState> =
        callbackFlowFromNotifier(this.onUiStateChange)
            .debounce(TUNNEL_STATE_UPDATE_DEBOUNCE_DURATION_MILLIS)

    private fun ConnectionProxy.tunnelRealStateFlow(): Flow<TunnelState> =
        callbackFlowFromNotifier(this.onStateChange)
            .debounce(TUNNEL_STATE_UPDATE_DEBOUNCE_DURATION_MILLIS)

    fun toggleTunnelInfoExpansion() {
        _isTunnelInfoExpanded.value = _isTunnelInfoExpanded.value.not()
    }

    fun tunnelUiState(): TunnelState = uiState.value.tunnelUiState

    companion object {
        const val TUNNEL_STATE_UPDATE_DEBOUNCE_DURATION_MILLIS: Long = 200
    }
}
