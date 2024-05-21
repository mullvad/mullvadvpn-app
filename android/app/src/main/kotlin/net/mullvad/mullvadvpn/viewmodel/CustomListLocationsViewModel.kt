package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.compose.state.CustomListLocationsUiState
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.descendants
import net.mullvad.mullvadvpn.relaylist.filterOnSearchTerm
import net.mullvad.mullvadvpn.relaylist.getById
import net.mullvad.mullvadvpn.usecase.RelayListUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.util.firstOrNullWithTimeout

class CustomListLocationsViewModel(
    private val customListId: CustomListId,
    private val newList: Boolean,
    private val relayListUseCase: RelayListUseCase,
    private val customListActionUseCase: CustomListActionUseCase
) : ViewModel() {
    private val _uiSideEffect =
        MutableSharedFlow<CustomListLocationsSideEffect>(replay = 1, extraBufferCapacity = 1)
    val uiSideEffect: SharedFlow<CustomListLocationsSideEffect> = _uiSideEffect

    private val _initialLocations = MutableStateFlow<Set<RelayItem>>(emptySet())
    private val _selectedLocations = MutableStateFlow<Set<RelayItem>?>(null)
    private val _searchTerm = MutableStateFlow(EMPTY_SEARCH_TERM)

    val uiState =
        combine(relayListUseCase.fullRelayList(), _searchTerm, _selectedLocations) {
                relayCountries,
                searchTerm,
                selectedLocations ->
                val filteredRelayCountries = relayCountries.filterOnSearchTerm(searchTerm, null)

                when {
                    selectedLocations == null ->
                        CustomListLocationsUiState.Loading(newList = newList)
                    filteredRelayCountries.isEmpty() ->
                        CustomListLocationsUiState.Content.Empty(
                            newList = newList,
                            searchTerm = searchTerm
                        )
                    else ->
                        CustomListLocationsUiState.Content.Data(
                            newList = newList,
                            searchTerm = searchTerm,
                            availableLocations = filteredRelayCountries,
                            selectedLocations = selectedLocations,
                            saveEnabled =
                                selectedLocations.isNotEmpty() &&
                                    selectedLocations != _initialLocations.value,
                            hasUnsavedChanges = selectedLocations != _initialLocations.value
                        )
                }
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                CustomListLocationsUiState.Loading(newList = newList)
            )

    init {
        viewModelScope.launch { fetchInitialSelectedLocations() }
    }

    fun save() {
        viewModelScope.launch {
            _selectedLocations.value?.let { selectedLocations ->
                customListActionUseCase
                    .performAction(
                        CustomListAction.UpdateLocations(
                            customListId,
                            selectedLocations.calculateLocationsToSave().mapNotNull {
                                it.location()
                            }
                        )
                    )
                    .fold(
                        { TODO("We probably should of handled this") },
                        {
                            _uiSideEffect.tryEmit(
                                // This is so that we don't show a snackbar after returning to
                                // the select
                                // location screen
                                if (newList) {
                                    CustomListLocationsSideEffect.CloseScreen
                                } else {
                                    CustomListLocationsSideEffect.ReturnWithResult(it)
                                }
                            )
                        }
                    )
            }
        }
    }

    fun onRelaySelectionClick(relayItem: RelayItem, selected: Boolean) {
        if (selected) {
            selectLocation(relayItem)
        } else {
            deselectLocation(relayItem)
        }
    }

    fun onSearchTermInput(searchTerm: String) {
        viewModelScope.launch { _searchTerm.emit(searchTerm) }
    }

    private suspend fun awaitCustomListById(id: CustomListId): RelayItem.CustomList? =
        relayListUseCase
            .customLists()
            .mapNotNull { customList -> customList.getById(id) }
            .firstOrNullWithTimeout(GET_CUSTOM_LIST_TIMEOUT_MS)

    private fun selectLocation(relayItem: RelayItem) {
        viewModelScope.launch {
            _selectedLocations.update {
                it?.plus(relayItem)?.plus(relayItem.descendants()) ?: setOf(relayItem)
            }
        }
    }

    private fun deselectLocation(relayItem: RelayItem) {
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

    private fun availableLocations(): List<RelayItem.Country> =
        (uiState.value as? CustomListLocationsUiState.Content.Data)?.availableLocations
            ?: emptyList()

    private fun Set<RelayItem>.deselectParents(relayItem: RelayItem): Set<RelayItem> {
        val availableLocations = availableLocations()
        val updateSelectionList = this.toMutableSet()
        when (relayItem) {
            is RelayItem.City -> {
                availableLocations
                    .find { it.code == relayItem.location.countryCode }
                    ?.let { updateSelectionList.remove(it) }
            }
            is RelayItem.Relay -> {
                availableLocations
                    .flatMap { country -> country.cities }
                    .find { it.code == relayItem.location.cityCode }
                    ?.let { updateSelectionList.remove(it) }
                availableLocations
                    .find { it.code == relayItem.location.countryCode }
                    ?.let { updateSelectionList.remove(it) }
            }
            is RelayItem.Country,
            is RelayItem.CustomList -> {
                /* Do nothing */
            }
        }

        return updateSelectionList
    }

    private fun Set<RelayItem>.calculateLocationsToSave(): List<RelayItem> {
        // We don't want to save children for a selected parent
        val saveSelectionList = this.toMutableList()
        this.forEach { relayItem ->
            when (relayItem) {
                is RelayItem.Country -> {
                    saveSelectionList.removeAll(relayItem.cities)
                    saveSelectionList.removeAll(relayItem.relays)
                }
                is RelayItem.City -> {
                    saveSelectionList.removeAll(relayItem.relays)
                }
                is RelayItem.Relay,
                is RelayItem.CustomList -> {
                    /* Do nothing */
                }
            }
        }
        return saveSelectionList
    }

    private fun List<RelayItem>.selectChildren(): Set<RelayItem> =
        (this + flatMap { it.descendants() }).toSet()

    private suspend fun fetchInitialSelectedLocations() {
        _selectedLocations.value =
            awaitCustomListById(customListId)?.locations?.selectChildren().apply {
                _initialLocations.value = this ?: emptySet()
            }
    }

    companion object {
        private const val EMPTY_SEARCH_TERM = ""
        private const val GET_CUSTOM_LIST_TIMEOUT_MS = 5000L
    }
}

sealed interface CustomListLocationsSideEffect {
    data object CloseScreen : CustomListLocationsSideEffect

    data class ReturnWithResult(val result: CustomListResult.LocationsChanged) :
        CustomListLocationsSideEffect
}
