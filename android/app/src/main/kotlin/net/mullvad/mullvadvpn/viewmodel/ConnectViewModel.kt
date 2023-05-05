package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.ui.fragment.ConnectFragment
import net.mullvad.mullvadvpn.ui.serviceconnection.LocationInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.util.appVersionCallbackFlow
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier

class ConnectViewModel(
    serviceConnectionManager: ServiceConnectionManager,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) : ViewModel() {
    private val _shared: SharedFlow<ServiceConnectionContainer>

    private val _isTunnelInfoExpanded = MutableStateFlow(false)
    private val _tunnelState: MutableStateFlow<Pair<TunnelState, TunnelState>> =
        MutableStateFlow(TunnelState.Disconnected to TunnelState.Disconnected)
    private val _location: MutableStateFlow<GeoIpLocation?> = MutableStateFlow(null)
    private val _relayLocation: MutableStateFlow<RelayItem?> = MutableStateFlow(null)
    private val _versionInfo: MutableStateFlow<VersionInfo?> = MutableStateFlow(null)
    val isTunnelInfoExpanded = _isTunnelInfoExpanded.asStateFlow()
    val tunnelState = _tunnelState.asStateFlow()
    val location = _location.asStateFlow()
    val relayLocation = _relayLocation.asStateFlow()
    val versionInfo = _versionInfo.asStateFlow()

    init {
        _shared =
            serviceConnectionManager.connectionState
                .flatMapLatest { state ->
                    if (state is ServiceConnectionState.ConnectedReady) {
                        flowOf(state.container)
                    } else {
                        emptyFlow()
                    }
                }
                .shareIn(viewModelScope, SharingStarted.WhileSubscribed())

        viewModelScope.launch(dispatcher) {
            launchLocationSubscription()
            launchRelayLocationSubscription()
            launchVersionInfoSubscription()
            launchTunnelStateSubscription()
        }
    }

    private fun CoroutineScope.launchLocationSubscription() = launch {
        _shared.flatMapLatest { it.locationInfoCache.locationCallbackFlow() }.collect(_location)
    }

    private fun LocationInfoCache.locationCallbackFlow() = callbackFlow {
        onNewLocation = { this.trySend(it) }
        awaitClose { onNewLocation = null }
    }

    private fun CoroutineScope.launchRelayLocationSubscription() = launch {
        _shared
            .flatMapLatest { it.relayListListener.relayListCallbackFlow() }
            .collect(_relayLocation)
    }

    private fun RelayListListener.relayListCallbackFlow() = callbackFlow {
        onRelayListChange = { _, item -> this.trySend(item) }
        awaitClose { onRelayListChange = null }
    }

    private fun CoroutineScope.launchVersionInfoSubscription() = launch {
        _shared
            .flatMapLatest { it.appVersionInfoCache.appVersionCallbackFlow() }
            .collect(_versionInfo)
    }

    private fun CoroutineScope.launchTunnelStateSubscription() = launch {
        _shared
            .flatMapLatest {
                combine(
                    callbackFlowFromNotifier(it.connectionProxy.onUiStateChange),
                    callbackFlowFromNotifier(it.connectionProxy.onStateChange)
                ) { uiState, realState ->
                    Pair(uiState, realState)
                }
            }
            // Fix to avoid wrong notification shown due to very frequent tunnel state updates.
            .debounce(ConnectFragment.TUNNEL_STATE_UPDATE_DEBOUNCE_DURATION_MILLIS)
            .collect(_tunnelState)
    }

    fun toggleTunnelInfoExpansion() {
        _isTunnelInfoExpanded.value = _isTunnelInfoExpanded.value.not()
    }
}
