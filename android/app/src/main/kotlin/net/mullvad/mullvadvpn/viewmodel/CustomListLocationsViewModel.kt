package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.getOrElse
import arrow.core.raise.either
import com.ramcosta.composedestinations.generated.destinations.CustomListLocationsDestination
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.communication.LocationsChanged
import net.mullvad.mullvadvpn.compose.state.CustomListLocationsUiState
import net.mullvad.mullvadvpn.compose.state.RelayLocationListItem
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.relaylist.ancestors
import net.mullvad.mullvadvpn.relaylist.descendants
import net.mullvad.mullvadvpn.relaylist.newFilterOnSearch
import net.mullvad.mullvadvpn.relaylist.withDescendants
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListRelayItemsUseCase

class CustomListLocationsViewModel(
    private val relayListRepository: RelayListRepository,
    private val customListRelayItemsUseCase: CustomListRelayItemsUseCase,
    private val customListActionUseCase: CustomListActionUseCase,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    private val navArgs =
        CustomListLocationsDestination.argsFrom(savedStateHandle = savedStateHandle)
    private val _uiSideEffect = Channel<CustomListLocationsSideEffect>()
    val uiSideEffect: Flow<CustomListLocationsSideEffect> = _uiSideEffect.receiveAsFlow()

    private val _initialLocations = MutableStateFlow<Set<RelayItem.Location>>(emptySet())
    private val _selectedLocations = MutableStateFlow<Set<RelayItem.Location>?>(null)
    private val _searchTerm = MutableStateFlow(EMPTY_SEARCH_TERM)
    private val _expandedItems = MutableStateFlow<Set<RelayItemId>>(setOf())

    val uiState =
        combine(searchRelayListLocations(), _searchTerm, _selectedLocations, _expandedItems) {
                relayCountries,
                searchTerm,
                selectedLocations,
                expandedLocations ->
                when {
                    selectedLocations == null ->
                        CustomListLocationsUiState.Loading(newList = navArgs.newList)
                    relayCountries.isEmpty() ->
                        CustomListLocationsUiState.Content.Empty(
                            newList = navArgs.newList,
                            searchTerm = searchTerm,
                            isSearching = searchTerm.length >= MIN_SEARCH_LENGTH,
                        )
                    else ->
                        CustomListLocationsUiState.Content.Data(
                            newList = navArgs.newList,
                            searchTerm = searchTerm,
                            locations =
                                relayCountries.toRelayItems(
                                    isSelected = { it in selectedLocations },
                                    isExpanded = { it in expandedLocations },
                                ),
                            saveEnabled =
                                selectedLocations.isNotEmpty() &&
                                    selectedLocations != _initialLocations.value,
                            hasUnsavedChanges = selectedLocations != _initialLocations.value,
                        )
                }
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                CustomListLocationsUiState.Loading(newList = navArgs.newList),
            )

    init {
        viewModelScope.launch { fetchInitialSelectedLocations() }
    }

    private fun searchRelayListLocations() =
        combine(_searchTerm, relayListRepository.relayList) { searchTerm, relayCountries ->
                val isSearching = searchTerm.length >= MIN_SEARCH_LENGTH
                if (isSearching) {
                    val (exp, filteredRelayCountries) = relayCountries.newFilterOnSearch(searchTerm)
                    exp.toSet() to filteredRelayCountries
                } else {
                    initialExpands(
                        _selectedLocations.value?.calculateLocationsToSave() ?: emptyList()
                    ) to relayCountries
                }
            }
            .onEach { _expandedItems.value = it.first }
            .map { it.second }

    fun save() {
        viewModelScope.launch {
            _selectedLocations.value?.let { selectedLocations ->
                val locationsToSave = selectedLocations.calculateLocationsToSave()
                val result =
                    either {
                            val success =
                                customListActionUseCase(
                                        CustomListAction.UpdateLocations(
                                            navArgs.customListId,
                                            locationsToSave.map { it.id },
                                        )
                                    )
                                    .bind()
                            calculateResultData(success, locationsToSave)
                        }
                        .getOrElse { CustomListActionResultData.GenericError }
                _uiSideEffect.send(
                    CustomListLocationsSideEffect.ReturnWithResultData(result = result)
                )
            }
        }
    }

    fun onRelaySelectionClick(relayItem: RelayItem.Location, selected: Boolean) {
        if (selected) {
            selectLocation(relayItem)
        } else {
            deselectLocation(relayItem)
        }
    }

    fun onExpand(relayItem: RelayItem.Location, expand: Boolean) {
        _expandedItems.update {
            if (expand) {
                it + relayItem.id
            } else {
                it - relayItem.id
            }
        }
    }

    fun onSearchTermInput(searchTerm: String) {
        viewModelScope.launch { _searchTerm.emit(searchTerm) }
    }

    private fun selectLocation(relayItem: RelayItem.Location) {
        viewModelScope.launch {
            _selectedLocations.update {
                it?.plus(relayItem)?.plus(relayItem.descendants()) ?: setOf(relayItem)
            }
        }
    }

    private fun deselectLocation(relayItem: RelayItem.Location) {
        viewModelScope.launch {
            _selectedLocations.update {
                val newSelectedLocations = it?.toMutableSet() ?: mutableSetOf()
                newSelectedLocations.remove(relayItem)
                newSelectedLocations.removeAll(relayItem.descendants().toSet())
                // If a parent is selected, deselect it, since we only want to select a parent if
                // all children are selected
                newSelectedLocations.deselectParents(relayItem)
            }
        }
    }

    private fun Set<RelayItem.Location>.deselectParents(
        relayItem: RelayItem.Location
    ): Set<RelayItem.Location> {
        val availableLocations = relayListRepository.relayList.value
        val updateSelectionList = this.toMutableSet()
        when (relayItem) {
            is RelayItem.Location.City -> {
                availableLocations
                    .find { it.id == relayItem.id.country }
                    ?.let { updateSelectionList.remove(it) }
            }
            is RelayItem.Location.Relay -> {
                availableLocations
                    .flatMap { country -> country.cities }
                    .find { it.id == relayItem.id.city }
                    ?.let { updateSelectionList.remove(it) }
                availableLocations
                    .find { it.id == relayItem.id.country }
                    ?.let { updateSelectionList.remove(it) }
            }
            is RelayItem.Location.Country -> {
                /* Do nothing */
            }
        }

        return updateSelectionList
    }

    private fun Set<RelayItem.Location>.calculateLocationsToSave(): List<RelayItem.Location> {
        // We don't want to save children for a selected parent
        val saveSelectionList = this.toMutableList()
        this.forEach { relayItem ->
            when (relayItem) {
                is RelayItem.Location.Country -> {
                    saveSelectionList.removeAll(relayItem.cities)
                    saveSelectionList.removeAll(relayItem.relays)
                }
                is RelayItem.Location.City -> {
                    saveSelectionList.removeAll(relayItem.relays)
                }
                is RelayItem.Location.Relay -> {
                    /* Do nothing */
                }
            }
        }
        return saveSelectionList
    }

    private suspend fun fetchInitialSelectedLocations() {
        val locations = customListRelayItemsUseCase(navArgs.customListId).first()
        val selectedLocations = locations.withDescendants().toSet()

        _initialLocations.value = selectedLocations
        _selectedLocations.value = selectedLocations
        // Initial expand
        _expandedItems.value = initialExpands(locations)
    }

    private fun initialExpands(locations: List<RelayItem.Location>): Set<RelayItemId> =
        locations.flatMap { it.id.ancestors() }.toSet()

    private fun List<RelayItem.Location>.toRelayItems(
        isSelected: (RelayItem) -> Boolean,
        isExpanded: (RelayItemId) -> Boolean,
        depth: Int = 0,
    ): List<RelayLocationListItem> = flatMap { relayItem ->
        buildList {
            val expanded = isExpanded(relayItem.id)
            add(
                RelayLocationListItem(
                    item = relayItem,
                    depth = depth,
                    checked = isSelected(relayItem),
                    expanded = expanded,
                )
            )
            if (expanded) {
                when (relayItem) {
                    is RelayItem.Location.City ->
                        addAll(
                            relayItem.relays.toRelayItems(
                                isSelected = isSelected,
                                isExpanded = isExpanded,
                                depth = depth + 1,
                            )
                        )
                    is RelayItem.Location.Country ->
                        addAll(
                            relayItem.cities.toRelayItems(
                                isSelected = isSelected,
                                isExpanded = isExpanded,
                                depth = depth + 1,
                            )
                        )
                    is RelayItem.Location.Relay -> {
                        /* Do nothing */
                    }
                }
            }
        }
    }

    private fun calculateResultData(
        success: LocationsChanged,
        locationsToSave: List<RelayItem.Location>,
    ) =
        if (navArgs.newList) {
            CustomListActionResultData.Success.CreatedWithLocations(
                customListName = success.name,
                locationNames = locationsToSave.map { it.name },
                undo = CustomListAction.Delete(id = success.id),
            )
        } else {
            when {
                success.addedLocations.size == 1 && success.removedLocations.isEmpty() ->
                    CustomListActionResultData.Success.LocationAdded(
                        customListName = success.name,
                        relayListRepository.find(success.addedLocations.first())!!.name,
                        undo = success.undo,
                    )
                success.removedLocations.size == 1 && success.addedLocations.isEmpty() ->
                    CustomListActionResultData.Success.LocationRemoved(
                        customListName = success.name,
                        locationName =
                            relayListRepository.find(success.removedLocations.first())!!.name,
                        undo = success.undo,
                    )
                else ->
                    CustomListActionResultData.Success.LocationChanged(
                        customListName = success.name,
                        undo = success.undo,
                    )
            }
        }

    companion object {
        private const val EMPTY_SEARCH_TERM = ""
        private const val MIN_SEARCH_LENGTH = 2
    }
}

sealed interface CustomListLocationsSideEffect {
    data class ReturnWithResultData(val result: CustomListActionResultData) :
        CustomListLocationsSideEffect
}
