package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.filterOnSearchTerm
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.usecase.RelayListUseCase

class SelectLocationViewModel(
    private val serviceConnectionManager: ServiceConnectionManager,
    private val relayListUseCase: RelayListUseCase
) : ViewModel() {
    private val _searchTerm = MutableStateFlow(EMPTY_SEARCH_TERM)

    val uiState =
        combine(relayListUseCase.relayListWithSelection(), _searchTerm) {
                (relayCountries, relayItem),
                searchTerm ->
                val filteredRelayCountries =
                    relayCountries.filterOnSearchTerm(searchTerm, relayItem)
                if (searchTerm.isNotEmpty() && filteredRelayCountries.isEmpty()) {
                    SelectLocationUiState.NoSearchResultFound(searchTerm = searchTerm)
                } else {
                    SelectLocationUiState.ShowData(
                        countries = filteredRelayCountries,
                        selectedRelay = relayItem
                    )
                }
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                SelectLocationUiState.Loading
            )

    private val _uiSideEffect = MutableSharedFlow<SelectLocationSideEffect>()
    val uiSideEffect = _uiSideEffect.asSharedFlow()

    fun selectRelay(relayItem: RelayItem) {
        relayListUseCase.updateSelectedRelayLocation(relayItem.location)
        serviceConnectionManager.connectionProxy()?.connect()
        viewModelScope.launch { _uiSideEffect.emit(SelectLocationSideEffect.CloseScreen) }
    }

    fun onSearchTermInput(searchTerm: String) {
        viewModelScope.launch { _searchTerm.emit(searchTerm) }
    }

    companion object {
        private const val EMPTY_SEARCH_TERM = ""
    }
}

sealed interface SelectLocationSideEffect {
    data object CloseScreen : SelectLocationSideEffect
}
