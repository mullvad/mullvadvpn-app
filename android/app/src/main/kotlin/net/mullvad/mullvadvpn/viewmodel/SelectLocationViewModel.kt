package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.relayListListener

class SelectLocationViewModel(private val serviceConnectionManager: ServiceConnectionManager) :
    ViewModel() {
    private val _closeAction = MutableSharedFlow<Unit>()

    val uiState =
        serviceConnectionManager.connectionState
            .flatMapLatest { state ->
                if (state is ServiceConnectionState.ConnectedReady) {
                    state.container.relayListListener.relayListCallbackFlow()
                } else {
                    emptyFlow()
                }
            }
            .map { (relayList, relayItem) ->
                SelectLocationUiState.ShowData(
                    countries = relayList.countries,
                    selectedRelay = relayItem
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                SelectLocationUiState.Loading
            )

    val uiCloseAction = _closeAction.asSharedFlow()

    fun selectRelay(relayItem: RelayItem?) {
        serviceConnectionManager.relayListListener()?.selectedRelayLocation = relayItem?.location
        serviceConnectionManager.connectionProxy()?.connect()
        viewModelScope.launch { _closeAction.emit(Unit) }
    }

    private fun RelayListListener.relayListCallbackFlow() = callbackFlow {
        onRelayListChange = { list, item -> this.trySend(list to item) }
        awaitClose { onRelayListChange = null }
    }
}
