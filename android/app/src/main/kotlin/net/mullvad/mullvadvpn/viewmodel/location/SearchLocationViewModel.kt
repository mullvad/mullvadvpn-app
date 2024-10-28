package net.mullvad.mullvadvpn.viewmodel.location

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.destinations.SearchLocationDestination
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.state.RelayListSelection
import net.mullvad.mullvadvpn.compose.state.SearchSelectLocationUiState
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.relaylist.MIN_SEARCH_LENGTH
import net.mullvad.mullvadvpn.relaylist.newFilterOnSearch
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.usecase.FilterChip
import net.mullvad.mullvadvpn.usecase.FilterChipUseCase
import net.mullvad.mullvadvpn.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.usecase.SelectedLocationUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.usecase.customlists.FilterCustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.util.combine

class SearchLocationViewModel(
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val relayListRepository: RelayListRepository,
    private val filteredRelayListUseCase: FilteredRelayListUseCase,
    private val customListActionUseCase: CustomListActionUseCase,
    private val customListsRepository: CustomListsRepository,
    private val relayListFilterRepository: RelayListFilterRepository,
    private val filterChipUseCase: FilterChipUseCase,
    filteredCustomListRelayItemsUseCase: FilterCustomListsRelayItemUseCase,
    selectedLocationUseCase: SelectedLocationUseCase,
    customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val relayListSelection: RelayListSelection =
        SearchLocationDestination.argsFrom(savedStateHandle).relayListSelection

    private val _searchTerm = MutableStateFlow(EMPTY_SEARCH_TERM)
    private val _expandedItems = MutableStateFlow<Set<String>>(emptySet())

    val uiState: StateFlow<SearchSelectLocationUiState> =
        combine(
                _searchTerm,
                searchRelayListLocations(),
                filteredCustomListRelayItemsUseCase(),
                customListsRelayItemUseCase(),
                selectedLocationUseCase(),
                filterChips(),
                _expandedItems,
            ) {
                searchTerm,
                relayCountries,
                filteredCustomLists,
                customLists,
                selectedItem,
                filterChips,
                expandedItems ->
                if (searchTerm.length >= MIN_SEARCH_LENGTH) {
                    SearchSelectLocationUiState.Content(
                        searchTerm = searchTerm,
                        relayListItems =
                            relayListItems(
                                searchTerm = searchTerm,
                                relayCountries = relayCountries,
                                customLists = filteredCustomLists,
                                selectedItem =
                                    selectedItem.getForRelayListSelect(relayListSelection),
                                disabledItem =
                                    selectedItem.getForRelayListDisabled(
                                        relayListSelection,
                                        customLists,
                                    ),
                                expandedItems = expandedItems,
                            ),
                        customLists = customLists,
                        relayListSelection = relayListSelection,
                        filterChips = filterChips,
                    )
                } else {
                    SearchSelectLocationUiState.NoQuery(searchTerm, filterChips)
                }
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                SearchSelectLocationUiState.NoQuery("", emptyList()),
            )

    private val _uiSideEffect = Channel<SearchLocationSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun onSearchInputUpdated(searchTerm: String) {
        viewModelScope.launch { _searchTerm.emit(searchTerm) }
    }

    fun selectRelay(relayItem: RelayItem) {
        viewModelScope.launch {
            selectRelayItem(
                    relayItem = relayItem,
                    relayListSelection = relayListSelection,
                    selectEntryLocation = wireguardConstraintsRepository::setEntryLocation,
                    selectExitLocation = relayListRepository::updateSelectedRelayLocation,
                )
                .fold(
                    { _uiSideEffect.send(SearchLocationSideEffect.GenericError) },
                    {
                        _uiSideEffect.send(
                            SearchLocationSideEffect.LocationSelected(relayListSelection)
                        )
                    },
                )
        }
    }

    private fun searchRelayListLocations() =
        combine(_searchTerm, filteredRelayListUseCase()) { searchTerm, relayCountries ->
                val (exp, filteredRelayCountries) = relayCountries.newFilterOnSearch(searchTerm)
                exp.map { it.expandKey() }.toSet() to filteredRelayCountries
            }
            .onEach { _expandedItems.value = it.first }
            .map { it.second }

    private fun filterChips() =
        filterChipUseCase().map { filterChips: List<FilterChip> ->
            filterChips.toMutableList().apply {
                // Do not show entry and exit filter chips if multihop is disabled
                if (multihopEnabled()) {
                    add(
                        when (relayListSelection) {
                            RelayListSelection.Entry -> FilterChip.Entry
                            RelayListSelection.Exit -> FilterChip.Exit
                        }
                    )
                }
            }
        }

    private fun multihopEnabled() =
        wireguardConstraintsRepository.wireguardConstraints.value?.useMultihop == true

    fun addLocationToList(item: RelayItem.Location, customList: RelayItem.CustomList) {
        viewModelScope.launch {
            val result =
                addLocationToCustomList(
                    item = item,
                    customList = customList,
                    update = customListActionUseCase::invoke,
                )
            _uiSideEffect.send(SearchLocationSideEffect.CustomListActionToast(result))
        }
    }

    fun removeLocationFromList(item: RelayItem.Location, customListId: CustomListId) {
        viewModelScope.launch {
            val result =
                removeLocationFromCustomList(
                    item = item,
                    customListId = customListId,
                    getCustomListById = customListsRepository::getCustomListById,
                    update = customListActionUseCase::invoke,
                )
            _uiSideEffect.trySend(SearchLocationSideEffect.CustomListActionToast(result))
        }
    }

    fun performAction(action: CustomListAction) {
        viewModelScope.launch { customListActionUseCase(action) }
    }

    fun removeOwnerFilter() {
        viewModelScope.launch { relayListFilterRepository.updateSelectedOwnership(Constraint.Any) }
    }

    fun removeProviderFilter() {
        viewModelScope.launch { relayListFilterRepository.updateSelectedProviders(Constraint.Any) }
    }

    fun onToggleExpand(item: RelayItemId, parent: CustomListId? = null, expand: Boolean) {
        _expandedItems.onToggleExpand(item = item, parent = parent, expand = expand)
    }

    companion object {
        private const val EMPTY_SEARCH_TERM = ""
    }
}

sealed interface SearchLocationSideEffect {
    data class LocationSelected(val relayListSelection: RelayListSelection) :
        SearchLocationSideEffect

    data class CustomListActionToast(val resultData: CustomListActionResultData) :
        SearchLocationSideEffect

    data object GenericError : SearchLocationSideEffect
}
