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
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.CustomListLocationsData
import net.mullvad.mullvadvpn.compose.state.CustomListLocationsUiState
import net.mullvad.mullvadvpn.lib.common.Lce
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.util.relaylist.ancestors
import net.mullvad.mullvadvpn.lib.common.util.relaylist.descendants
import net.mullvad.mullvadvpn.lib.common.util.relaylist.newFilterOnSearch
import net.mullvad.mullvadvpn.lib.common.util.relaylist.withDescendants
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.communication.CustomListAction
import net.mullvad.mullvadvpn.lib.model.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.lib.model.communication.LocationsChanged
import net.mullvad.mullvadvpn.lib.repository.RelayListRepository
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.CheckableRelayListItem
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.ItemPosition
import net.mullvad.mullvadvpn.lib.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.lib.usecase.customlists.CustomListRelayItemsUseCase

@Suppress("TooManyFunctions")
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
    private val _expandOverrides = MutableStateFlow<Map<RelayItemId, Boolean>>(mapOf())

    val uiState =
        combine(_searchTerm, relayListRepository.relayList, _selectedLocations, _expandOverrides) {
                searchTerm,
                relayCountries,
                selectedLocations,
                expandOverrides ->
                when {
                    selectedLocations == null ->
                        CustomListLocationsUiState(
                            newList = navArgs.newList,
                            content = Lce.Loading(Unit),
                        )

                    relayCountries.isEmpty() ->
                        CustomListLocationsUiState(
                            newList = navArgs.newList,
                            content = Lce.Error(Unit),
                        )

                    else -> {
                        val (expandSet, filteredRelayCountries) =
                            searchRelayListLocations(searchTerm, relayCountries)
                        val expandedLocations = expandSet.with(expandOverrides)
                        CustomListLocationsUiState(
                            newList = navArgs.newList,
                            content =
                                Lce.Content(
                                    CustomListLocationsData(
                                        searchTerm = searchTerm,
                                        locations =
                                            filteredRelayCountries.flatMap {
                                                it.toRelayItems(
                                                    isSelected = { it in selectedLocations },
                                                    isExpanded = { it in expandedLocations },
                                                    isLastChild = true,
                                                )
                                            },
                                        saveEnabled =
                                            selectedLocations.isNotEmpty() &&
                                                selectedLocations != _initialLocations.value,
                                        hasUnsavedChanges =
                                            selectedLocations != _initialLocations.value,
                                    )
                                ),
                        )
                    }
                }
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                CustomListLocationsUiState(newList = navArgs.newList, content = Lce.Loading(Unit)),
            )

    init {
        viewModelScope.launch { fetchInitialSelectedLocations() }
    }

    private fun searchRelayListLocations(
        searchTerm: String,
        relayCountries: List<RelayItem.Location.Country>,
    ) =
        if (searchTerm.isNotEmpty()) {
            val (exp, filteredRelayCountries) = relayCountries.newFilterOnSearch(searchTerm)
            exp.toSet() to filteredRelayCountries
        } else {
            initialExpands(_selectedLocations.value?.calculateLocationsToSave() ?: emptyList()) to
                relayCountries
        }

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
        _expandOverrides.update { it + (relayItem.id to expand) }
    }

    fun onSearchTermInput(searchTerm: String) {
        viewModelScope.launch {
            _expandOverrides.emit(emptyMap())
            _searchTerm.emit(searchTerm)
        }
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
        _expandOverrides.value = initialExpands(locations).associateWith { true }
    }

    private fun initialExpands(locations: List<RelayItem.Location>): Set<RelayItemId> =
        locations.flatMap { it.id.ancestors() }.toSet()

    private fun RelayItem.Location.toRelayItems(
        isSelected: (RelayItem) -> Boolean,
        isExpanded: (RelayItemId) -> Boolean,
        depth: Int = 0,
        isLastChild: Boolean,
    ): List<CheckableRelayListItem> = buildList {
        val expanded = isExpanded(id)
        add(
            CheckableRelayListItem(
                item = this@toRelayItems,
                depth = depth,
                checked = isSelected(this@toRelayItems),
                expanded = expanded,
                itemPosition =
                    when {
                        this@toRelayItems is RelayItem.Location.Country ->
                            if (!expanded) ItemPosition.Single else ItemPosition.Top
                        isLastChild && !expanded -> ItemPosition.Bottom
                        else -> ItemPosition.Middle
                    },
            )
        )
        if (expanded) {
            when (this@toRelayItems) {
                is RelayItem.Location.City ->
                    addAll(
                        relays.flatMapIndexed { index, relay ->
                            relay.toRelayItems(
                                isSelected = isSelected,
                                isExpanded = isExpanded,
                                depth = depth + 1,
                                isLastChild = isLastChild && index == relays.lastIndex,
                            )
                        }
                    )

                is RelayItem.Location.Country ->
                    addAll(
                        cities.flatMapIndexed { index, item ->
                            item.toRelayItems(
                                isSelected = isSelected,
                                isExpanded = isExpanded,
                                depth = depth + 1,
                                isLastChild = isLastChild && index == cities.lastIndex,
                            )
                        }
                    )

                is RelayItem.Location.Relay -> {
                    /* Do nothing */
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

    private fun Set<RelayItemId>.with(overrides: Map<RelayItemId, Boolean>): Set<RelayItemId> =
        this + overrides.filterValues { expanded -> expanded }.keys -
            overrides.filterValues { expanded -> !expanded }.keys

    companion object {
        private const val EMPTY_SEARCH_TERM = ""
    }
}

sealed interface CustomListLocationsSideEffect {
    data class ReturnWithResultData(val result: CustomListActionResultData) :
        CustomListLocationsSideEffect
}
