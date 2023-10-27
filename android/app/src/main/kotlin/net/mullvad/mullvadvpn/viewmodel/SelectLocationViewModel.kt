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
import net.mullvad.mullvadvpn.relaylist.filterOnSearchTerm
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.relayListListener

class SelectLocationViewModel(private val serviceConnectionManager: ServiceConnectionManager) :
    ViewModel() {
    private val _closeAction = MutableSharedFlow<Unit>()
    private val _enterTransitionEndAction = MutableSharedFlow<Unit>()
    private val _searchTerm = MutableStateFlow(EMPTY_SEARCH_TERM)

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
                combine(serviceConnection.relayListListener.relayListCallbackFlow(), _searchTerm) {
                    (relayCountries, relayItem),
                    searchTerm ->
                    Triple(
                        relayCountries.filterOnSearchTerm(searchTerm, relayItem),
                        relayItem,
                        searchTerm
                    )
                }
            }
            .map { (relayCountries, relayItem, searchTerm) ->
                if (searchTerm.isNotEmpty() && relayCountries.isEmpty()) {
                    SelectLocationUiState.NoSearchResultFound(searchTerm = searchTerm)
                } else {
                    SelectLocationUiState.ShowData(
                        countries = relayCountries,
                        selectedRelay = relayItem
                    )
                }
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                SelectLocationUiState.Loading
            )

    @Suppress("konsist.ensure public properties use permitted names")
    val uiCloseAction = _closeAction.asSharedFlow()
    @Suppress("konsist.ensure public properties use permitted names")
    val enterTransitionEndAction = _enterTransitionEndAction.asSharedFlow()

    fun selectRelay(relayItem: RelayItem) {
        serviceConnectionManager
            .relayListListener()
            ?.updateSelectedRelayLocation(relayItem.location)
        serviceConnectionManager.connectionProxy()?.connect()
        viewModelScope.launch { _closeAction.emit(Unit) }
    }

    fun onTransitionAnimationEnd() {
        viewModelScope.launch { _enterTransitionEndAction.emit(Unit) }
    }

    fun onSearchTermInput(searchTerm: String) {
        viewModelScope.launch { _searchTerm.emit(searchTerm) }
    }

    private fun RelayListListener.relayListCallbackFlow() = callbackFlow {
        onRelayCountriesChange = { list, item -> this.trySend(list to item) }
        awaitClose { onRelayCountriesChange = null }
    }

    companion object {
        private const val EMPTY_SEARCH_TERM = ""
    }
}
