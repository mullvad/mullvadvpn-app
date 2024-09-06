package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.getOrElse
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
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
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
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.usecase.AvailableProvidersUseCase
import net.mullvad.mullvadvpn.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.usecase.customlists.FilterCustomListsRelayItemUseCase

class SelectLocationViewModel(
    private val relayListFilterRepository: RelayListFilterRepository,
    private val availableProvidersUseCase: AvailableProvidersUseCase,
    customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    private val filteredCustomListRelayItemsUseCase: FilterCustomListsRelayItemUseCase,
    private val customListsRepository: CustomListsRepository,
    private val customListActionUseCase: CustomListActionUseCase,
    private val filteredRelayListUseCase: FilteredRelayListUseCase,
    private val relayListRepository: RelayListRepository,
    private val settingsRepository: SettingsRepository,
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
                    customLists = customLists,
                )
            }
            .stateIn(viewModelScope, SharingStarted.Lazily, SelectLocationUiState.Loading)

    private val _uiSideEffect = Channel<SelectLocationSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private fun initialExpand(): Set<String> = buildSet {
        when (val item = relayListRepository.selectedLocation.value.getOrNull()) {
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

    private fun filterChips() =
        combine(
            relayListFilterRepository.selectedOwnership,
            relayListFilterRepository.selectedProviders,
            availableProvidersUseCase(),
            settingsRepository.settingsUpdates,
        ) { selectedOwnership, selectedConstraintProviders, allProviders, settings ->
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
            buildList {
                if (ownershipFilter != null) {
                    add(FilterChip.Ownership(ownershipFilter))
                }
                if (providerCountFilter != null) {
                    add(FilterChip.Provider(providerCountFilter))
                }
                if (settings?.isDaitaEnabled() == true) {
                    add(FilterChip.Daita)
                }
            }
        }

    private fun relayListItems() =
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
                    ) {
                        it in expandedItems
                    }
                if (relayItems.isEmpty()) {
                    add(RelayListItem.LocationsEmptyText(searchTerm))
                } else {
                    addAll(relayItems)
                }
            }
        }

    private fun createRelayListItems(
        isSearching: Boolean,
        selectedItem: RelayItemId?,
        customLists: List<RelayItem.CustomList>,
        countries: List<RelayItem.Location.Country>,
        isExpanded: (String) -> Boolean,
    ): List<RelayListItem> =
        createCustomListSection(isSearching, selectedItem, customLists, isExpanded) +
            createLocationSection(isSearching, selectedItem, countries, isExpanded)

    private fun createCustomListSection(
        isSearching: Boolean,
        selectedItem: RelayItemId?,
        customLists: List<RelayItem.CustomList>,
        isExpanded: (String) -> Boolean,
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

    private fun createCustomListRelayItems(
        customLists: List<RelayItem.CustomList>,
        selectedItem: RelayItemId?,
        isExpanded: (String) -> Boolean,
    ): List<RelayListItem> =
        customLists.flatMap { customList ->
            val expanded = isExpanded(customList.id.expandKey())
            buildList {
                add(
                    RelayListItem.CustomListItem(
                        customList,
                        isSelected = selectedItem == customList.id,
                        expanded,
                    )
                )

                if (expanded) {
                    addAll(
                        customList.locations.flatMap {
                            createCustomListEntry(parent = customList, item = it, 1, isExpanded)
                        }
                    )
                }
            }
        }

    private fun createLocationSection(
        isSearching: Boolean,
        selectedItem: RelayItemId?,
        countries: List<RelayItem.Location.Country>,
        isExpanded: (String) -> Boolean,
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

    private fun createCustomListEntry(
        parent: RelayItem.CustomList,
        item: RelayItem.Location,
        depth: Int = 1,
        isExpanded: (String) -> Boolean,
    ): List<RelayListItem.CustomListEntryItem> = buildList {
        val expanded = isExpanded(item.id.expandKey(parent.id))
        add(
            RelayListItem.CustomListEntryItem(
                parentId = parent.id,
                parentName = parent.customList.name,
                item = item,
                expanded = expanded,
                depth,
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

    private fun createGeoLocationEntry(
        item: RelayItem.Location,
        selectedItem: RelayItemId?,
        depth: Int = 0,
        isExpanded: (String) -> Boolean,
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
                    { _uiSideEffect.send(SelectLocationSideEffect.GenericError) },
                    { _uiSideEffect.send(SelectLocationSideEffect.CloseScreen) },
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
        selectedOwnership: Ownership?,
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
            val result =
                customListActionUseCase(
                        CustomListAction.UpdateLocations(customList.id, newLocations)
                    )
                    .fold(
                        { CustomListActionResultData.GenericError },
                        {
                            if (it.removedLocations.isEmpty()) {
                                CustomListActionResultData.Success.LocationAdded(
                                    customListName = it.name,
                                    locationName = item.name,
                                    undo = it.undo,
                                )
                            } else {
                                CustomListActionResultData.Success.LocationChanged(
                                    customListName = it.name,
                                    undo = it.undo,
                                )
                            }
                        },
                    )
            _uiSideEffect.send(SelectLocationSideEffect.CustomListActionToast(result))
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
                        val success =
                            customListActionUseCase(
                                    CustomListAction.UpdateLocations(customList.id, newLocations)
                                )
                                .bind()
                        if (success.addedLocations.isEmpty()) {
                            CustomListActionResultData.Success.LocationRemoved(
                                customListName = success.name,
                                locationName = item.name,
                                undo = success.undo,
                            )
                        } else {
                            CustomListActionResultData.Success.LocationChanged(
                                customListName = success.name,
                                undo = success.undo,
                            )
                        }
                    }
                    .getOrElse { CustomListActionResultData.GenericError }
            _uiSideEffect.send(SelectLocationSideEffect.CustomListActionToast(result))
        }
    }

    companion object {
        private const val EMPTY_SEARCH_TERM = ""
    }
}

sealed interface SelectLocationSideEffect {
    data object CloseScreen : SelectLocationSideEffect

    data class CustomListActionToast(val resultData: CustomListActionResultData) :
        SelectLocationSideEffect

    data object GenericError : SelectLocationSideEffect
}
