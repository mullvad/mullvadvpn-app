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
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.compose.state.ConnectNotificationState
import net.mullvad.mullvadvpn.compose.state.ConnectUiState
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.ui.VersionInfo
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
import org.joda.time.DateTime

class ConnectViewModel(
    private val serviceConnectionManager: ServiceConnectionManager,
    private val isVersionInfoNotificationEnabled: Boolean =
        BuildConfig.ENABLE_IN_APP_VERSION_NOTIFICATIONS,
    accountRepository: AccountRepository
) : ViewModel() {
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
                    accountRepository.accountExpiryState,
                    _isTunnelInfoExpanded
                ) {
                    location,
                    relayLocation,
                    versionInfo,
                    tunnelUiState,
                    tunnelRealState,
                    accountExpiry,
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
                        connectNotificationState =
                            calculateNotificationState(
                                tunnelUiState = tunnelUiState,
                                versionInfo = versionInfo,
                                accountExpiry = accountExpiry
                            )
                    )
                }
            }
            .debounce(UI_STATE_DEBOUNCE_DURATION_MILLIS)
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), ConnectUiState.INITIAL)

    private fun LocationInfoCache.locationCallbackFlow() =
        callbackFlow {
                onNewLocation = { this.trySend(it) }
                awaitClose { onNewLocation = null }
            }
            // Filter out empty or short-name country representations.
            .filter { it?.let { location -> location.country.length > 2 } ?: false }

    private fun RelayListListener.relayListCallbackFlow() = callbackFlow {
        onRelayCountriesChange = { _, item -> this.trySend(item) }
        awaitClose { onRelayCountriesChange = null }
    }

    private fun ConnectionProxy.tunnelUiStateFlow(): Flow<TunnelState> =
        callbackFlowFromNotifier(this.onUiStateChange)

    private fun ConnectionProxy.tunnelRealStateFlow(): Flow<TunnelState> =
        callbackFlowFromNotifier(this.onStateChange)

    private fun calculateNotificationState(
        tunnelUiState: TunnelState,
        versionInfo: VersionInfo?,
        accountExpiry: AccountExpiry
    ): ConnectNotificationState =
        when {
            tunnelUiState is TunnelState.Connecting ->
                ConnectNotificationState.ShowTunnelStateNotificationBlocked
            tunnelUiState is TunnelState.Disconnecting &&
                (tunnelUiState.actionAfterDisconnect == ActionAfterDisconnect.Block ||
                    tunnelUiState.actionAfterDisconnect == ActionAfterDisconnect.Reconnect) ->
                ConnectNotificationState.ShowTunnelStateNotificationBlocked
            tunnelUiState is TunnelState.Error ->
                ConnectNotificationState.ShowTunnelStateNotificationError(tunnelUiState.errorState)
            isVersionInfoNotificationEnabled &&
                versionInfo != null &&
                (versionInfo.isOutdated || !versionInfo.isSupported) ->
                ConnectNotificationState.ShowVersionInfoNotification(versionInfo)
            accountExpiry.shouldShowNotification() ->
                ConnectNotificationState.ShowAccountExpiryNotification(
                    accountExpiry.date() ?: DateTime.now()
                )
            else -> ConnectNotificationState.HideNotification
        }

    private fun AccountExpiry.shouldShowNotification(): Boolean {
        val threeDaysFromNow = DateTime.now().plusDays(3)
        return this.date()?.isBefore(threeDaysFromNow) == true
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

    companion object {
        const val UI_STATE_DEBOUNCE_DURATION_MILLIS: Long = 200
    }
}
