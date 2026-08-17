package net.mullvad.mullvadvpn.feature.location.impl.search

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.feature.location.impl.onToggleExpandMap
import net.mullvad.mullvadvpn.lib.common.Lce
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.util.combine
import net.mullvad.mullvadvpn.lib.common.util.ignoreEntrySelection
import net.mullvad.mullvadvpn.lib.common.util.relaylist.RelayMetadata
import net.mullvad.mullvadvpn.lib.common.util.relaylist.RelayMetadataMap
import net.mullvad.mullvadvpn.lib.common.util.relaylist.filterOnSearchTerm
import net.mullvad.mullvadvpn.lib.common.util.relaylist.merge
import net.mullvad.mullvadvpn.lib.common.util.relaylist.newFilterOnSearch
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayHopType
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelayListSearchResult
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.model.SearchMatch
import net.mullvad.mullvadvpn.lib.model.communication.CustomListAction
import net.mullvad.mullvadvpn.lib.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository
import net.mullvad.mullvadvpn.lib.usecase.FilterChip
import net.mullvad.mullvadvpn.lib.usecase.FilterChipUseCase
import net.mullvad.mullvadvpn.lib.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.lib.usecase.ModifyMultihopError
import net.mullvad.mullvadvpn.lib.usecase.ModifyMultihopUseCase
import net.mullvad.mullvadvpn.lib.usecase.MultihopChange
import net.mullvad.mullvadvpn.lib.usecase.SelectRelayItemError
import net.mullvad.mullvadvpn.lib.usecase.SelectSinglehopUseCase
import net.mullvad.mullvadvpn.lib.usecase.SelectedLocationUseCase
import net.mullvad.mullvadvpn.lib.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.lib.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.lib.usecase.customlists.FilterCustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.lib.usecase.itemOrNull

@Suppress("LongParameterList", "TooManyFunctions")
class SearchLocationViewModel(
    private val relayListType: RelayListType,
    private val customListActionUseCase: CustomListActionUseCase,
    private val relayListFilterRepository: RelayListFilterRepository,
    private val filterChipUseCase: FilterChipUseCase,
    private val selectSinglehopUseCase: SelectSinglehopUseCase,
    private val modifyMultihopUseCase: ModifyMultihopUseCase,
    private val settingsRepository: SettingsRepository,
    filteredRelayListUseCase: FilteredRelayListUseCase,
    filteredCustomListRelayItemsUseCase: FilterCustomListsRelayItemUseCase,
    selectedLocationUseCase: SelectedLocationUseCase,
    customListsRelayItemUseCase: CustomListsRelayItemUseCase,
) : ViewModel() {

    private val _searchTerm = MutableStateFlow(EMPTY_SEARCH_TERM)
    private val _expandOverrides = MutableStateFlow<Map<String, Boolean>>(emptyMap())

    val uiState: StateFlow<Lce<Unit, SearchLocationUiState, Unit>> =
        combine(
                _searchTerm,
                filteredRelayListUseCase(relayListType),
                filteredCustomListRelayItemsUseCase(relayListType = relayListType),
                customListsRelayItemUseCase(),
                selectedLocationUseCase(),
                filterChips(),
                _expandOverrides,
            ) {
                searchTerm,
                filteredCountries,
                filteredCustomLists,
                customLists,
                selectedItem,
                filterChips,
                expandOverrides ->
                if (filteredCountries.countries.isEmpty()) {
                    return@combine Lce.Error(Unit)
                }
                val relaySearch =
                    searchRelayListLocations(
                        searchTerm = searchTerm,
                        relayCountries = filteredCountries.countries,
                    )
                val expandSet = relaySearch.expansionSet.map { it.expandKey() }.toSet()
                val expandedItems = expandSet.with(expandOverrides)
                val customListSearch = filteredCustomLists.filterOnSearchTerm(searchTerm)
                val allHighlights = relaySearch.highlights + customListSearch.highlights
                val metadata = filteredCountries.relayMetadata.addHighlights(allHighlights)
                val settings = settingsRepository.settingsUpdates.value
                Lce.Content(
                    SearchLocationUiState(
                        searchTerm = searchTerm,
                        relayListType = relayListType,
                        relayListItems =
                            relayListItemsSearching(
                                searchTerm = searchTerm,
                                relayCountries = relaySearch.matchedCountries,
                                relayMetadata = metadata,
                                relayListType = relayListType,
                                customLists = customListSearch.matchedCustomLists,
                                selectedByThisEntryExitList =
                                    selectedItem.selectedByThisEntryExitList(relayListType),
                                selectedByOtherEntryExitList =
                                    if (
                                        ignoreEntrySelection(
                                            settings,
                                            relayListType,
                                        )
                                    ) {
                                        null
                                    } else {
                                        selectedItem.selectedByOtherEntryExitList(
                                            relayListType,
                                            customLists,
                                        )
                                    },
                                expandedItems = expandedItems,
                            ),
                        customLists = customLists,
                        filterChips = filterChips,
                    )
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lce.Loading(Unit),
            )

    private val _uiSideEffect = Channel<SearchLocationSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun onSearchInputUpdated(searchTerm: String) {
        viewModelScope.launch {
            _expandOverrides.emit(emptyMap())
            _searchTerm.emit(searchTerm)
        }
    }

    fun selectRelayItem(relayItem: RelayItem, relayListType: RelayListType) {
        viewModelScope.launch {
            when (relayListType) {
                is RelayListType.Multihop ->
                    modifyMultihop(
                        when (relayListType.hopType) {
                            RelayHopType.ENTRY -> MultihopChange.Entry(Constraint.Only(relayItem))
                            RelayHopType.EXIT -> MultihopChange.Exit(relayItem)
                        }
                    )

                RelayListType.Single -> selectSinglehop(item = relayItem)
            }
        }
    }

    fun selectAutomaticMultihopEntry() {
        viewModelScope.launch { modifyMultihop(MultihopChange.Entry(Constraint.Any)) }
    }

    private suspend fun selectSinglehop(item: RelayItem) =
        selectSinglehopUseCase(item)
            .fold(
                { _uiSideEffect.send(it.toSideEffect()) },
                { _uiSideEffect.send(SearchLocationSideEffect.LocationSelected(relayListType)) },
            )

    private suspend fun modifyMultihop(change: MultihopChange) =
        modifyMultihopUseCase(change = change)
            .fold(
                {
                    change.itemOrNull()?.let { changedItem ->
                        _uiSideEffect.send(
                            it.toSideEffect(change = change, changedItem = changedItem)
                        )
                    }
                },
                { _uiSideEffect.send(SearchLocationSideEffect.LocationSelected(relayListType)) },
            )

    private fun searchRelayListLocations(
        searchTerm: String,
        relayCountries: List<RelayItem.Location.Country>,
    ): RelayListSearchResult =
        if (searchTerm.isNotEmpty()) {
            relayCountries.newFilterOnSearch(searchTerm)
        } else {
            RelayListSearchResult(
                matchedCountries = relayCountries,
                expansionSet = emptySet(),
                highlights = emptyMap(),
            )
        }

    private fun filterChips() =
        filterChipUseCase(relayListType).map { filterChips ->
            filterChips.toMutableList().apply {
                // Only show entry and exit filter chips if relayListType is Multihop
                if (relayListType is RelayListType.Multihop) {
                    when (relayListType.hopType) {
                        RelayHopType.ENTRY -> add(FilterChip.Entry)
                        RelayHopType.EXIT -> add(FilterChip.Exit)
                    }
                }
            }
        }

    fun performAction(action: CustomListAction) {
        viewModelScope.launch { customListActionUseCase(action) }
    }

    fun removeOwnerFilter(filterTarget: RelayHopType) {
        viewModelScope.launch {
            relayListFilterRepository.updateSelectedOwnership(Constraint.Any, filterTarget)
        }
    }

    fun removeProviderFilter(filterTarget: RelayHopType) {
        viewModelScope.launch {
            relayListFilterRepository.updateSelectedProviders(Constraint.Any, filterTarget)
        }
    }

    fun onToggleExpand(item: RelayItemId, parent: CustomListId? = null, expand: Boolean) {
        _expandOverrides.onToggleExpandMap(item = item, parent = parent, expand = expand)
    }

    private fun Set<String>.with(overrides: Map<String, Boolean>): Set<String> =
        this + overrides.filterValues { expanded -> expanded }.keys -
            overrides.filterValues { expanded -> !expanded }.keys

    private fun ModifyMultihopError.toSideEffect(
        change: MultihopChange,
        changedItem: RelayItem,
    ): SearchLocationSideEffect =
        when (this) {
            is ModifyMultihopError.EntrySameAsExit ->
                when (change) {
                    is MultihopChange.Entry ->
                        SearchLocationSideEffect.ExitAlreadySelected(relayItem = changedItem)

                    is MultihopChange.Exit ->
                        SearchLocationSideEffect.EntryAlreadySelected(relayItem = changedItem)
                }

            ModifyMultihopError.GenericError -> SearchLocationSideEffect.GenericError
            is ModifyMultihopError.RelayItemInactive ->
                SearchLocationSideEffect.RelayItemInactive(relayItem = this.relayItem)
        }

    private fun SelectRelayItemError.toSideEffect() =
        when (this) {
            SelectRelayItemError.EntryAndExitSame ->
                error("Entry and exit should not be the same when using Single hop")

            SelectRelayItemError.GenericError -> SearchLocationSideEffect.GenericError
            is SelectRelayItemError.RelayInactive ->
                SearchLocationSideEffect.RelayItemInactive(this.relayItem)
        }

    companion object {
        private const val EMPTY_SEARCH_TERM = ""
    }
}

private fun RelayMetadataMap.addHighlights(
    highlights: Map<RelayItemId, SearchMatch>
): RelayMetadataMap {
    val highlightsMetadata = highlights.mapValues { (_, match) ->
        RelayMetadata(titleHighlights = match.matchRange)
    }
    return merge(highlightsMetadata)
}

sealed interface SearchLocationSideEffect {
    data class LocationSelected(val relayListType: RelayListType) : SearchLocationSideEffect

    data class RelayItemInactive(val relayItem: RelayItem) : SearchLocationSideEffect

    data class EntryAlreadySelected(val relayItem: RelayItem) : SearchLocationSideEffect

    data class ExitAlreadySelected(val relayItem: RelayItem) : SearchLocationSideEffect

    data object GenericError : SearchLocationSideEffect
}
