package net.mullvad.mullvadvpn.viewmodel

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.CustomListLocationsUiState
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.filterOnSearchTerm
import net.mullvad.mullvadvpn.usecase.CustomListUseCase
import net.mullvad.mullvadvpn.usecase.RelayListUseCase

class CustomListLocationsViewModel(
    private val customListId: String,
    relayListUseCase: RelayListUseCase,
    private val customListUseCase: CustomListUseCase
) : ViewModel() {
    private var customListName: String = ""

    private val _uiSideEffect =
        MutableSharedFlow<CustomListLocationsSideEffect>(replay = 1, extraBufferCapacity = 1)
    val uiSideEffect: SharedFlow<CustomListLocationsSideEffect> = _uiSideEffect

    private val _selectedLocations = MutableStateFlow<Set<RelayItem>?>(null)
    private val _searchTerm = MutableStateFlow(EMPTY_SEARCH_TERM)

    val uiState =
        combine(relayListUseCase.relayList(), _searchTerm, _selectedLocations) {
                relayCountries,
                searchTerm,
                selectedLocations ->
                val filteredRelayCountries = relayCountries.filterOnSearchTerm(searchTerm, null)

                when {
                    selectedLocations == null -> CustomListLocationsUiState.Loading
                    filteredRelayCountries.isEmpty() ->
                        CustomListLocationsUiState.Content.Empty(searchTerm)
                    else ->
                        CustomListLocationsUiState.Content.Data(
                            searchTerm = searchTerm,
                            availableLocations = filteredRelayCountries,
                            selectedLocations = selectedLocations,
                        )
                }
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                CustomListLocationsUiState.Loading,
            )

    init {
        viewModelScope.launch {
            _selectedLocations.value =
                relayListUseCase
                    .customLists()
                    .firstOrNull()
                    ?.firstOrNull { it.id == customListId }
                    ?.apply { customListName = name }
                    ?.locations
                    ?.selectChildren()
        }
    }

    fun selectLocation(relayItem: RelayItem) {
        viewModelScope.launch {
            _selectedLocations.update {
                val newSelectedLocations = it?.toMutableSet() ?: mutableSetOf()
                newSelectedLocations.add(relayItem)
                when (relayItem) {
                    is RelayItem.Country -> {
                        newSelectedLocations.addAll(relayItem.cities)
                        newSelectedLocations.addAll(relayItem.relays)
                    }
                    is RelayItem.City -> {
                        newSelectedLocations.addAll(relayItem.relays)
                    }
                    is RelayItem.Relay -> {
                        /* Do nothing */
                    }
                    is RelayItem.CustomList ->
                        throw IllegalArgumentException("CustomList not supported")
                }
                // Select parent if all children are selected
                newSelectedLocations.selectParents(relayItem)
            }
        }
    }

    fun deselectLocation(relayItem: RelayItem) {
        viewModelScope.launch {
            _selectedLocations.update {
                val newSelectedLocations = it?.toMutableSet() ?: mutableSetOf()
                newSelectedLocations.remove(relayItem)
                when (relayItem) {
                    is RelayItem.Country -> {
                        newSelectedLocations.removeAll(relayItem.cities.toSet())
                        newSelectedLocations.removeAll(relayItem.relays.toSet())
                    }
                    is RelayItem.City -> {
                        newSelectedLocations.removeAll(relayItem.relays.toSet())
                    }
                    is RelayItem.Relay -> {
                        /* Do nothing */
                    }
                    is RelayItem.CustomList ->
                        throw IllegalArgumentException("CustomList not supported")
                }
                // If a parent is selected, deselect it, since we only want to select a parent if
                // all children are selected
                newSelectedLocations.deselectParents(relayItem)
            }
        }
    }

    fun save() {
        viewModelScope.launch {
            _selectedLocations.value?.let { selectedLocations ->
                customListUseCase.updateCustomListLocations(
                    id = customListId,
                    locations = selectedLocations.calculateLocationsToSave()
                )
                _uiSideEffect.tryEmit(CustomListLocationsSideEffect.CloseScreen)
            }
        }
    }

    private fun availableLocations(): List<RelayItem.Country> =
        (uiState.value as? CustomListLocationsUiState.Content.Data)?.availableLocations
            ?: emptyList()

    private fun Set<RelayItem>.selectParents(relayItem: RelayItem): Set<RelayItem> {
        val availableLocations = availableLocations()
        val updateSelectionList = this.toMutableSet()
        when (relayItem) {
            is RelayItem.City -> {
                val country = availableLocations.find { it.code == relayItem.location.countryCode }
                if (country != null && updateSelectionList.containsAll(country.cities)) {
                    updateSelectionList.add(country)
                }
            }
            is RelayItem.Relay -> {
                val city =
                    availableLocations
                        .flatMap { country -> country.cities }
                        .find { it.code == relayItem.location.cityCode }
                Log.d("LOLZ", "city: $city code=${relayItem.location.cityCode}")
                if (city != null && updateSelectionList.containsAll(city.relays)) {
                    updateSelectionList.add(city)
                    val country = availableLocations.find { it.code == city.location.countryCode }
                    if (country != null && updateSelectionList.containsAll(country.cities)) {
                        updateSelectionList.add(country)
                    }
                }
            }
            is RelayItem.Country,
            is RelayItem.CustomList -> {
                /* Do nothing */
            }
        }

        return updateSelectionList
    }

    private fun Set<RelayItem>.deselectParents(relayItem: RelayItem): Set<RelayItem> {
        val availableLocations = availableLocations()
        val updateSelectionList = this.toMutableSet()
        Log.d("LOLZ", "availableLocations: $availableLocations")
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

    private fun List<RelayItem>.selectChildren(): Set<RelayItem> {
        val selectedLocations = this.toMutableSet()
        forEach { relayItem ->
            when (relayItem) {
                is RelayItem.Country -> {
                    selectedLocations.addAll(relayItem.cities)
                    selectedLocations.addAll(relayItem.relays)
                }
                is RelayItem.City -> {
                    selectedLocations.addAll(relayItem.relays)
                }
                is RelayItem.Relay,
                is RelayItem.CustomList -> {
                    /* Do nothing */
                }
            }
        }
        return selectedLocations
    }

    companion object {
        private const val EMPTY_SEARCH_TERM = ""
    }
}

sealed interface CustomListLocationsSideEffect {
    data object CloseScreen : CustomListLocationsSideEffect
}
