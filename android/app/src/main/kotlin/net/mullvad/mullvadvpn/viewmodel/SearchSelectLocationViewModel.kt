package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.destinations.SelectLocationSearchDestination
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.RelayListSelection
import net.mullvad.mullvadvpn.compose.state.SearchSelectLocationUiState
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.relaylist.MIN_SEARCH_LENGTH
import net.mullvad.mullvadvpn.relaylist.newFilterOnSearch
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository

class SearchSelectLocationViewModel(
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val relayListRepository: RelayListRepository,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val relayListSelection: RelayListSelection =
        SelectLocationSearchDestination.argsFrom(savedStateHandle).relayListSelection

    private val _searchTerm = MutableStateFlow(EMPTY_SEARCH_TERM)

    val uiState: StateFlow<SearchSelectLocationUiState> =
        _searchTerm
            .map { searchTerm ->
                if (searchTerm.length >= MIN_SEARCH_LENGTH) {
                    SearchSelectLocationUiState.Content(
                        searchTerm = searchTerm,
                        relayListItems = emptyList(),
                        relayListSelection = relayListSelection,
                    )
                } else {
                    SearchSelectLocationUiState.NoQuery(searchTerm)
                }
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                SearchSelectLocationUiState.NoQuery(""),
            )

    private val _uiSideEffect = Channel<SearchSelectLocationSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun selectRelay(relayItem: RelayItem) {
        viewModelScope.launch {
            val locationConstraint = relayItem.id
            when (relayListSelection) {
                RelayListSelection.Entry -> selectEntryLocation(locationConstraint)
                RelayListSelection.Exit -> selectExitLocation(locationConstraint)
            }.fold(
                { _uiSideEffect.send(SearchSelectLocationSideEffect.GenericError) },
                { _uiSideEffect.send(SearchSelectLocationSideEffect.CloseScreen) },
            )
        }
    }

    private suspend fun selectEntryLocation(locationConstraint: RelayItemId) =
        wireguardConstraintsRepository.setEntryLocation(locationConstraint)

    private suspend fun selectExitLocation(locationConstraint: RelayItemId) =
        relayListRepository.updateSelectedRelayLocation(locationConstraint)

    fun onToggleExpand(item: RelayItemId, parent: CustomListId? = null, expand: Boolean) {
        /*_expandedItems.update {
            val key = item.expandKey(parent)
            if (expand) {
                it + key
            } else {
                it - key
            }
        }*/
    }

    fun onSearchInputUpdated(searchTerm: String) {
        viewModelScope.launch { _searchTerm.emit(searchTerm) }
    }

    private fun searchRelayListLocations() =
        combine(_searchTerm, filteredRelayListUseCase()) { searchTerm, relayCountries ->
            val isSearching = searchTerm.length >= MIN_SEARCH_LENGTH
            if (isSearching) {
                val (exp, filteredRelayCountries) = relayCountries.newFilterOnSearch(searchTerm)
                exp.map { it.expandKey() }.toSet() to filteredRelayCountries
            } else {
                initialExpand() to relayCountries
            }
        }
            .onEach { _expandedItems.value = it.first }
            .map { it.second }

    companion object {
        private const val EMPTY_SEARCH_TERM = ""
    }
}

sealed interface SearchSelectLocationSideEffect {
    data object CloseScreen : SearchSelectLocationSideEffect

    data object GenericError : SearchSelectLocationSideEffect
}
