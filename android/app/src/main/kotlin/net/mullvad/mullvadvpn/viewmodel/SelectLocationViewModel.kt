package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.raise.either
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.LocationsChanged
import net.mullvad.mullvadvpn.compose.state.RelayListItem
import net.mullvad.mullvadvpn.compose.state.RelayListItem.CustomListHeader
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.state.toNullableOwnership
import net.mullvad.mullvadvpn.compose.state.toSelectedProviders
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.relaylist.MIN_SEARCH_LENGTH
import net.mullvad.mullvadvpn.relaylist.descendants
import net.mullvad.mullvadvpn.relaylist.filterOnOwnershipAndProvider
import net.mullvad.mullvadvpn.relaylist.filterOnSearchTerm
import net.mullvad.mullvadvpn.relaylist.newFilterOnSearch
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.usecase.AvailableProvidersUseCase
import net.mullvad.mullvadvpn.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.util.combine

class SelectLocationViewModel(
    private val relayListFilterRepository: RelayListFilterRepository,
    availableProvidersUseCase: AvailableProvidersUseCase,
    customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    private val customListsRepository: CustomListsRepository,
    private val customListActionUseCase: CustomListActionUseCase,
    filteredRelayListUseCase: FilteredRelayListUseCase,
    private val relayListRepository: RelayListRepository
) : ViewModel() {
    private val _searchTerm = MutableStateFlow(EMPTY_SEARCH_TERM)

    private fun initialExpand(): Set<String> {
        val item = relayListRepository.selectedLocation.value.getOrNull()
        return when (item) {
            is CustomListId -> setOf()
            is GeoLocationId.City -> setOf(item.countryCode.countryCode)
            is GeoLocationId.Country -> setOf()
            is GeoLocationId.Hostname -> setOf(item.country.countryCode, item.city.cityCode)
            null -> setOf()
        }
    }

    private val _expandedItems = MutableStateFlow(initialExpand())
    private val _searchExpandedItems = MutableStateFlow(setOf<String>())
    private val _searchCollapsedItems = MutableStateFlow(setOf<String>())

    @Suppress("DestructuringDeclarationWithTooManyEntries")
    val uiState =
        combine(
                filteredRelayListUseCase(),
                customListsRelayItemUseCase(),
                relayListRepository.selectedLocation,
                _expandedItems,
                _searchExpandedItems,
                _searchCollapsedItems,
                _searchTerm,
                relayListFilterRepository.selectedOwnership,
                availableProvidersUseCase(),
                relayListFilterRepository.selectedProviders,
            ) {
                relayCountries,
                customLists,
                selectedItem,
                expandedItems,
                searchExpandedItems,
                searchCollapsedItems,
                searchTerm,
                selectedOwnership,
                allProviders,
                selectedConstraintProviders ->
                val selectRelayItemId = selectedItem.getOrNull()
                val selectedOwnershipItem = selectedOwnership.toNullableOwnership()
                val selectedProvidersCount =
                    when (selectedConstraintProviders) {
                        is Constraint.Any -> null
                        is Constraint.Only ->
                            filterSelectedProvidersByOwnership(
                                    selectedConstraintProviders.toSelectedProviders(allProviders),
                                    selectedOwnershipItem,
                                )
                                .size
                    }

                val isSearching = searchTerm.length >= MIN_SEARCH_LENGTH
                val (expansionList, relayCountries1) =
                    if (isSearching) {
                        val (exp, filteredRelayCountries) =
                            relayCountries.newFilterOnSearch(searchTerm)
                        (searchExpandedItems + exp.map { it.expandKey() } - searchCollapsedItems) to
                            filteredRelayCountries
                    } else {
                        expandedItems to relayCountries
                    }

                val filteredCustomLists =
                    customLists
                        .filterOnSearchTerm(searchTerm)
                        .filterOnOwnershipAndProvider(
                            ownership = selectedOwnership,
                            providers = selectedConstraintProviders,
                        )

                SelectLocationUiState.Content(
                    searchTerm = searchTerm,
                    selectedOwnership = selectedOwnershipItem,
                    selectedProvidersCount = selectedProvidersCount,
                    relayListItems =
                        createRelayListItems(
                            isSearching,
                            selectedItem.getOrNull(),
                            filteredCustomLists,
                            relayCountries1,
                            expansionList),
                    customLists = customLists,
                    selectedItem = selectRelayItemId,
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                SelectLocationUiState.Loading,
            )

    private val _uiSideEffect = Channel<SelectLocationSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun createRelayListItems(
        isSearching: Boolean,
        selectedItem: RelayItemId?,
        customLists: List<RelayItem.CustomList>,
        countries: List<RelayItem.Location.Country>,
        expandedkeys: Set<String>
    ): List<RelayListItem> {

        val customListItems: List<RelayListItem> =
            if (isSearching && customLists.isEmpty()) emptyList()
            else {
                listOf(CustomListHeader) +
                    customLists.flatMap { customList ->
                        val expanded = customList.id.expandKey() in expandedkeys
                        val item =
                            listOf(
                                RelayListItem.CustomListItem(
                                    customList,
                                    isSelected = selectedItem == customList.id,
                                    expanded))

                        if (expanded) {
                            item +
                                customList.locations.flatMap {
                                    createCustomListEntry(
                                        parent = customList.id, item = it, 1, expandedkeys)
                                }
                        } else {
                            item
                        }
                    }
            }
        val relayLocations: List<RelayListItem> =
            countries.flatMap { country ->
                createGeoLocationEntry(country, selectedItem, expandedkeys = expandedkeys)
            }

        return customListItems + listOf(
            RelayListItem.CustomListFooter(customListItems.isNotEmpty()),
            RelayListItem.LocationHeader) + relayLocations
    }

    fun createCustomListEntry(
        parent: CustomListId,
        item: RelayItem.Location,
        depth: Int = 1,
        expandedkeys: Set<String>
    ): List<RelayListItem.CustomListEntryItem> {
        val expanded = item.id.expandKey(parent) in expandedkeys
        val entry =
            listOf(
                RelayListItem.CustomListEntryItem(
                    parentId = parent, item = item, expanded = expanded, depth))

        return if (expanded) {
            entry +
                when (item) {
                    is RelayItem.Location.City ->
                        item.relays.flatMap {
                            createCustomListEntry(parent, it, depth + 1, expandedkeys)
                        }
                    is RelayItem.Location.Country ->
                        item.cities.flatMap {
                            createCustomListEntry(parent, it, depth + 1, expandedkeys)
                        }
                    is RelayItem.Location.Relay -> emptyList<RelayListItem.CustomListEntryItem>()
                }
        } else {
            entry
        }
    }

    fun createGeoLocationEntry(
        item: RelayItem.Location,
        selectedItem: RelayItemId?,
        depth: Int = 0,
        expandedkeys: Set<String>
    ): List<RelayListItem.GeoLocationItem> {
        val expanded = item.id.expandKey() in expandedkeys
        val entry =
            listOf(
                RelayListItem.GeoLocationItem(
                    item = item,
                    isSelected = selectedItem == item.id,
                    depth = depth,
                    expanded = expanded,
                ))

        return if (expanded) {
            entry +
                when (item) {
                    is RelayItem.Location.City ->
                        item.relays.flatMap {
                            createGeoLocationEntry(it, selectedItem, depth + 1, expandedkeys)
                        }
                    is RelayItem.Location.Country ->
                        item.cities.flatMap {
                            createGeoLocationEntry(it, selectedItem, depth + 1, expandedkeys)
                        }
                    is RelayItem.Location.Relay -> emptyList()
                }
        } else {
            entry
        }
    }

    private fun RelayItemId.expandKey(parent: CustomListId? = null) =
        (parent?.value ?: "") +
            when (this) {
                is CustomListId -> value
                is GeoLocationId.City -> cityCode
                is GeoLocationId.Country -> countryCode
                is GeoLocationId.Hostname -> hostname
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
        if (_searchTerm.value.length >= MIN_SEARCH_LENGTH) {
            val key = item.expandKey(parent)
            if (expand) {
                _searchExpandedItems.update { it + key }
                _searchCollapsedItems.update { it - key }
            } else {
                _searchExpandedItems.update { it - key }
                _searchCollapsedItems.update { it + key }
            }
        } else {
            _expandedItems.update {
                val key = item.expandKey(parent)
                if (expand) {
                    it + key
                } else {
                    it - key
                }
            }
        }
    }

    fun onSearchTermInput(searchTerm: String) {
        viewModelScope.launch {
            _searchExpandedItems.update { setOf() }
            _searchCollapsedItems.update { setOf() }
            _searchTerm.emit(searchTerm)
        }
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
                                CustomListAction.UpdateLocations(customList.id, newLocations))
                            .bind()
                    }
                    .fold(
                        { SelectLocationSideEffect.GenericError },
                        { SelectLocationSideEffect.LocationRemovedFromCustomList(it) })
            _uiSideEffect.send(result)
        }
    }

    private fun List<RelayItem.CustomList>.filterOnOwnershipAndProvider(
        ownership: Constraint<Ownership>,
        providers: Constraint<Providers>
    ): List<RelayItem.CustomList> = map { item ->
        item.filterOnOwnershipAndProvider(ownership, providers)
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
}
