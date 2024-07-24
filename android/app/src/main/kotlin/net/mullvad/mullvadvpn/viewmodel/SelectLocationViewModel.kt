package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.raise.either
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.LocationsChanged
import net.mullvad.mullvadvpn.compose.state.FilterChip
import net.mullvad.mullvadvpn.compose.state.RelayListItem
import net.mullvad.mullvadvpn.compose.state.RelayListItem.CustomListHeader
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState.Content
import net.mullvad.mullvadvpn.compose.state.toSelectedProviders
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.relaylist.MIN_SEARCH_LENGTH
import net.mullvad.mullvadvpn.relaylist.descendants
import net.mullvad.mullvadvpn.relaylist.filterOnSearchTerm
import net.mullvad.mullvadvpn.relaylist.newFilterOnSearch
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.usecase.AvailableProvidersUseCase
import net.mullvad.mullvadvpn.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.usecase.customlists.FilterCustomListsRelayItemUseCase

class SelectLocationViewModel(
    private val relayListFilterRepository: RelayListFilterRepository,
    private val availableProvidersUseCase: AvailableProvidersUseCase,
    private val customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    private val filteredCustomListRelayItemsUseCase: FilterCustomListsRelayItemUseCase,
    private val customListsRepository: CustomListsRepository,
    private val customListActionUseCase: CustomListActionUseCase,
    private val filteredRelayListUseCase: FilteredRelayListUseCase,
    private val relayListRepository: RelayListRepository
) : ViewModel() {
    private val _searchTerm = MutableStateFlow(EMPTY_SEARCH_TERM)

    private val _expandedItems = MutableStateFlow(initialExpand())

    @Suppress("DestructuringDeclarationWithTooManyEntries")
    val uiState =
        combine(_searchTerm, relayListItems(), filterChips(), customListsRelayItemUseCase()) {
                searchTerm,
                relayListItems,
                filterChips,
                customLists ->
                Content(
                    searchTerm = searchTerm,
                    filterChips = filterChips,
                    relayListItems = relayListItems,
                    customLists = customLists
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                SelectLocationUiState.Loading,
            )

    private val _uiSideEffect = Channel<SelectLocationSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun centerOnSelected() =
        viewModelScope.launch {
            val selectedLocation = relayListRepository.selectedLocation.value.getOrNull()
            if (selectedLocation != null) {
                _uiSideEffect.send(SelectLocationSideEffect.CenterOnItem(selectedLocation))
            }
        }

    private fun initialExpand(): Set<String> = buildSet {
        val item = relayListRepository.selectedLocation.value.getOrNull()
        when (item) {
            is GeoLocationId.City -> {
                add(item.country.code)
            }
            is GeoLocationId.Hostname -> {
                add(item.country.code)
                add(item.city.code)
            }
            is CustomListId,
            is GeoLocationId.Country,
            null -> {
                /* No expands */
            }
        }
    }

    fun searchRelayListLocations() =
        combine(
                _searchTerm,
                filteredRelayListUseCase(),
            ) { searchTerm, relayCountries ->
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

    fun filterChips() =
        combine(
            relayListFilterRepository.selectedOwnership,
            relayListFilterRepository.selectedProviders,
            availableProvidersUseCase(),
        ) { selectedOwnership, selectedConstraintProviders, allProviders,
            ->
            val ownershipFilter = selectedOwnership.getOrNull()
            val providerCountFilter =
                when (selectedConstraintProviders) {
                    is Constraint.Any -> null
                    is Constraint.Only ->
                        filterSelectedProvidersByOwnership(
                                selectedConstraintProviders.toSelectedProviders(allProviders),
                                ownershipFilter,
                            )
                            .size
                }

            buildList<FilterChip> {
                if (ownershipFilter != null) {
                    add(FilterChip.Ownership(ownershipFilter))
                }
                if (providerCountFilter != null) {
                    add(FilterChip.Provider(providerCountFilter))
                }
            }
        }

    fun relayListItems() =
        combine(
            _searchTerm,
            searchRelayListLocations(),
            filteredCustomListRelayItemsUseCase(),
            relayListRepository.selectedLocation,
            _expandedItems,
        ) { searchTerm, relayCountries, customLists, selectedItem, expandedItems ->
            val filteredCustomLists = customLists.filterOnSearchTerm(searchTerm)

            buildList {
                val relayItems =
                    createRelayListItems(
                        searchTerm.length >= MIN_SEARCH_LENGTH,
                        selectedItem.getOrNull(),
                        filteredCustomLists,
                        relayCountries,
                        { it in expandedItems }
                    )
                if (relayItems.isEmpty()) {
                    add(RelayListItem.LocationsEmptyText(searchTerm))
                } else {
                    addAll(relayItems)
                }
            }
        }

    fun createRelayListItems(
        isSearching: Boolean,
        selectedItem: RelayItemId?,
        customLists: List<RelayItem.CustomList>,
        countries: List<RelayItem.Location.Country>,
        isExpanded: (String) -> Boolean
    ): List<RelayListItem> =
        createCustomListSection(isSearching, selectedItem, customLists, isExpanded) +
            createLocationSection(isSearching, selectedItem, countries, isExpanded)

    fun createCustomListSection(
        isSearching: Boolean,
        selectedItem: RelayItemId?,
        customLists: List<RelayItem.CustomList>,
        isExpanded: (String) -> Boolean
    ): List<RelayListItem> = buildList {
        if (isSearching && customLists.isEmpty()) {
            // If we are searching and no results are found don't show header or footer
        } else {
            add(CustomListHeader)
            val customListItems = createCustomListRelayItems(customLists, selectedItem, isExpanded)
            addAll(customListItems)
            add(RelayListItem.CustomListFooter(customListItems.isNotEmpty()))
        }
    }

    fun createCustomListRelayItems(
        customLists: List<RelayItem.CustomList>,
        selectedItem: RelayItemId?,
        isExpanded: (String) -> Boolean
    ): List<RelayListItem> =
        customLists.flatMap { customList ->
            val expanded = isExpanded(customList.id.expandKey())
            buildList<RelayListItem> {
                add(
                    RelayListItem.CustomListItem(
                        customList,
                        isSelected = selectedItem == customList.id,
                        expanded
                    )
                )

                if (expanded) {
                    addAll(
                        customList.locations.flatMap {
                            createCustomListEntry(parent = customList.id, item = it, 1, isExpanded)
                        }
                    )
                }
            }
        }

    fun createLocationSection(
        isSearching: Boolean,
        selectedItem: RelayItemId?,
        countries: List<RelayItem.Location.Country>,
        isExpanded: (String) -> Boolean
    ): List<RelayListItem> = buildList {
        if (isSearching && countries.isEmpty()) {
            // If we are searching and no results are found don't show header or footer
        } else {
            add(RelayListItem.LocationHeader)
            addAll(
                countries.flatMap { country ->
                    createGeoLocationEntry(country, selectedItem, isExpanded = isExpanded)
                }
            )
        }
    }

    fun createCustomListEntry(
        parent: CustomListId,
        item: RelayItem.Location,
        depth: Int = 1,
        isExpanded: (String) -> Boolean,
    ): List<RelayListItem.CustomListEntryItem> =
        buildList<RelayListItem.CustomListEntryItem> {
            val expanded = isExpanded(item.id.expandKey(parent))
            add(
                RelayListItem.CustomListEntryItem(
                    parentId = parent,
                    item = item,
                    expanded = expanded,
                    depth
                )
            )

            if (expanded) {
                when (item) {
                    is RelayItem.Location.City ->
                        addAll(
                            item.relays.flatMap {
                                createCustomListEntry(parent, it, depth + 1, isExpanded)
                            }
                        )
                    is RelayItem.Location.Country ->
                        addAll(
                            item.cities.flatMap {
                                createCustomListEntry(parent, it, depth + 1, isExpanded)
                            }
                        )
                    is RelayItem.Location.Relay -> {} // No children to add
                }
            }
        }

    fun createGeoLocationEntry(
        item: RelayItem.Location,
        selectedItem: RelayItemId?,
        depth: Int = 0,
        isExpanded: (String) -> Boolean
    ): List<RelayListItem.GeoLocationItem> = buildList {
        val expanded = isExpanded(item.id.expandKey())

        add(
            RelayListItem.GeoLocationItem(
                item = item,
                isSelected = selectedItem == item.id,
                depth = depth,
                expanded = expanded,
            )
        )

        if (expanded) {
            when (item) {
                is RelayItem.Location.City ->
                    addAll(
                        item.relays.flatMap {
                            createGeoLocationEntry(it, selectedItem, depth + 1, isExpanded)
                        }
                    )
                is RelayItem.Location.Country ->
                    addAll(
                        item.cities.flatMap {
                            createGeoLocationEntry(it, selectedItem, depth + 1, isExpanded)
                        }
                    )
                is RelayItem.Location.Relay -> {} // Do nothing
            }
        }
    }

    private fun RelayItemId.expandKey(parent: CustomListId? = null) =
        (parent?.value ?: "") +
            when (this) {
                is CustomListId -> value
                is GeoLocationId -> code
            }

    fun selectRelay(relayItem: RelayItem) {
        viewModelScope.launch {
            val locationConstraint = relayItem.id
            relayListRepository
                .updateSelectedRelayLocation(locationConstraint)
                .fold(
                    { _uiSideEffect.trySend(SelectLocationSideEffect.GenericError) },
                    { _uiSideEffect.trySend(SelectLocationSideEffect.CloseScreen) },
                )
        }
    }

    fun onToggleExpand(item: RelayItemId, parent: CustomListId? = null, expand: Boolean) {
        _expandedItems.update {
            val key = item.expandKey(parent)
            if (expand) {
                it + key
            } else {
                it - key
            }
        }
    }

    fun onSearchTermInput(searchTerm: String) {
        viewModelScope.launch { _searchTerm.emit(searchTerm) }
    }

    private fun filterSelectedProvidersByOwnership(
        selectedProviders: List<Provider>,
        selectedOwnership: Ownership?
    ): List<Provider> =
        if (selectedOwnership == null) selectedProviders
        else selectedProviders.filter { it.ownership == selectedOwnership }

    fun removeOwnerFilter() {
        viewModelScope.launch { relayListFilterRepository.updateSelectedOwnership(Constraint.Any) }
    }

    fun removeProviderFilter() {
        viewModelScope.launch { relayListFilterRepository.updateSelectedProviders(Constraint.Any) }
    }

    fun addLocationToList(item: RelayItem.Location, customList: RelayItem.CustomList) {
        viewModelScope.launch {
            val newLocations =
                (customList.locations + item).filter { it !in item.descendants() }.map { it.id }
            customListActionUseCase(CustomListAction.UpdateLocations(customList.id, newLocations))
                .fold(
                    { _uiSideEffect.send(SelectLocationSideEffect.GenericError) },
                    { _uiSideEffect.send(SelectLocationSideEffect.LocationAddedToCustomList(it)) },
                )
        }
    }

    fun performAction(action: CustomListAction) {
        viewModelScope.launch { customListActionUseCase(action) }
    }

    fun removeLocationFromList(item: RelayItem.Location, customListId: CustomListId) {
        viewModelScope.launch {
            val result =
                either {
                        val customList =
                            customListsRepository.getCustomListById(customListId).bind()
                        val newLocations = (customList.locations - item.id)

                        customListActionUseCase(
                                CustomListAction.UpdateLocations(customList.id, newLocations)
                            )
                            .bind()
                    }
                    .fold(
                        { SelectLocationSideEffect.GenericError },
                        { SelectLocationSideEffect.LocationRemovedFromCustomList(it) }
                    )
            _uiSideEffect.send(result)
        }
    }

    companion object {
        private const val EMPTY_SEARCH_TERM = ""
    }
}

sealed interface SelectLocationSideEffect {
    data object CloseScreen : SelectLocationSideEffect

    data class LocationAddedToCustomList(val result: LocationsChanged) : SelectLocationSideEffect

    class LocationRemovedFromCustomList(val result: LocationsChanged) : SelectLocationSideEffect

    data object GenericError : SelectLocationSideEffect

    data class CenterOnItem(val selectedItem: RelayItemId?) : SelectLocationSideEffect
}
