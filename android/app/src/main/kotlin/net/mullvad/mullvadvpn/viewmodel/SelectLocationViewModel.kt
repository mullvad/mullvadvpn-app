package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
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
    private val _filter = MutableStateFlow("")

    val uiState =
        serviceConnectionManager.connectionState
            .flatMapLatest { state ->
                if (state is ServiceConnectionState.ConnectedReady) {
                    flowOf(state.container)
                } else {
                    emptyFlow()
                }
            }
            .flatMapLatest { serviceConnection ->
                combine(serviceConnection.relayListListener.relayListCallbackFlow(), _filter) {
                    (relayList, relayItem),
                    filter ->
                    Triple(relayList.filter(filter, relayItem), relayItem, filter)
                }
            }
            .map { (relayList, relayItem, filter) ->
                if (filter.isNotEmpty() && relayList.countries.isEmpty()) {
                    SelectLocationUiState.NoSearchResultFound(searchTerm = filter)
                } else {
                    SelectLocationUiState.ShowData(
                        countries = relayList.countries,
                        selectedRelay = relayItem
                    )
                }
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

    fun onSearchRelays(filter: String) {
        viewModelScope.launch { _filter.emit(filter) }
    }

    private fun RelayListListener.relayListCallbackFlow() = callbackFlow {
        onRelayListChange = { list, item -> this.trySend(list to item) }
        awaitClose { onRelayListChange = null }
    }
}
