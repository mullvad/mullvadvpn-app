package net.mullvad.mullvadvpn.viewmodel.location

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.destinations.SearchLocationDestination
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.state.SearchLocationUiState
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.MultihopRelayListType
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.relaylist.newFilterOnSearch
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.usecase.FilterChip
import net.mullvad.mullvadvpn.usecase.FilterChipUseCase
import net.mullvad.mullvadvpn.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.usecase.ModifyMultihopError
import net.mullvad.mullvadvpn.usecase.ModifyMultihopUseCase
import net.mullvad.mullvadvpn.usecase.MultihopChange
import net.mullvad.mullvadvpn.usecase.SelectHopError
import net.mullvad.mullvadvpn.usecase.SelectHopUseCase
import net.mullvad.mullvadvpn.usecase.SelectedLocationUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.usecase.customlists.FilterCustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.util.Lce
import net.mullvad.mullvadvpn.util.combine

@Suppress("LongParameterList")
class SearchLocationViewModel(
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val customListActionUseCase: CustomListActionUseCase,
    private val customListsRepository: CustomListsRepository,
    private val relayListFilterRepository: RelayListFilterRepository,
    private val filterChipUseCase: FilterChipUseCase,
    private val selectHopUseCase: SelectHopUseCase,
    private val modifyMultihopUseCase: ModifyMultihopUseCase,
    private val settingsRepository: SettingsRepository,
    filteredRelayListUseCase: FilteredRelayListUseCase,
    filteredCustomListRelayItemsUseCase: FilterCustomListsRelayItemUseCase,
    selectedLocationUseCase: SelectedLocationUseCase,
    customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val relayListType: RelayListType =
        SearchLocationDestination.argsFrom(savedStateHandle).relayListType

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
                relayCountries,
                filteredCustomLists,
                customLists,
                selectedItem,
                filterChips,
                expandOverrides ->
                if (relayCountries.isEmpty()) {
                    return@combine Lce.Error(Unit)
                }
                val (expandSet, relayListLocations) =
                    searchRelayListLocations(
                        searchTerm = searchTerm,
                        relayCountries = relayCountries,
                    )
                val expandedItems = expandSet.with(expandOverrides)
                val settings = settingsRepository.settingsUpdates.value
                Lce.Content(
                    SearchLocationUiState(
                        searchTerm = searchTerm,
                        relayListType = relayListType,
                        relayListItems =
                            relayListItemsSearching(
                                searchTerm = searchTerm,
                                relayCountries = relayListLocations,
                                relayListType = relayListType,
                                customLists = filteredCustomLists,
                                selectedByThisEntryExitList =
                                    selectedItem.selectedByThisEntryExitList(relayListType),
                                selectedByOtherEntryExitList =
                                    if (ignoreEntrySelection(settings, relayListType)) {
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
                        selection = selectedItem,
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
                        when (relayListType.multihopRelayListType) {
                            MultihopRelayListType.ENTRY -> MultihopChange.Entry(relayItem)
                            MultihopRelayListType.EXIT -> MultihopChange.Exit(relayItem)
                        }
                    )
                RelayListType.Single -> selectHop(hop = Hop.Single(relayItem))
            }
        }
    }

    private suspend fun selectHop(hop: Hop.Single<*>) =
        selectHopUseCase(hop)
            .fold(
                {
                    _uiSideEffect.send(
                        when (it) {
                            SelectHopError.EntryAndExitSame ->
                                error("Entry and exit should not be the same when using Single hop")
                            SelectHopError.GenericError -> SearchLocationSideEffect.GenericError
                            is SelectHopError.HopInactive ->
                                SearchLocationSideEffect.RelayItemInactive(hop.relay)
                        }
                    )
                },
                { _uiSideEffect.send(SearchLocationSideEffect.LocationSelected(relayListType)) },
            )

    private suspend fun modifyMultihop(change: MultihopChange) =
        modifyMultihopUseCase(change = change)
            .fold(
                {
                    _uiSideEffect.send(
                        when (it) {
                            is ModifyMultihopError.EntrySameAsExit ->
                                when (change) {
                                    is MultihopChange.Entry ->
                                        SearchLocationSideEffect.ExitAlreadySelected(
                                            relayItem = change.item
                                        )
                                    is MultihopChange.Exit ->
                                        SearchLocationSideEffect.EntryAlreadySelected(
                                            relayItem = change.item
                                        )
                                }
                            ModifyMultihopError.GenericError ->
                                SearchLocationSideEffect.GenericError
                            is ModifyMultihopError.RelayItemInactive ->
                                SearchLocationSideEffect.RelayItemInactive(relayItem = it.relayItem)
                        }
                    )
                },
                { _uiSideEffect.send(SearchLocationSideEffect.LocationSelected(relayListType)) },
            )

    private fun searchRelayListLocations(
        searchTerm: String,
        relayCountries: List<RelayItem.Location.Country>,
    ) =
        if (searchTerm.isNotEmpty()) {
            val (exp, filteredRelayCountries) = relayCountries.newFilterOnSearch(searchTerm)
            exp.map { it.expandKey() }.toSet() to filteredRelayCountries
        } else {
            emptySet<String>() to relayCountries
        }

    private fun filterChips() =
        combine(
            filterChipUseCase(relayListType),
            wireguardConstraintsRepository.wireguardConstraints,
        ) { filterChips, constraints ->
            filterChips.toMutableList().apply {
                // Only show entry and exit filter chips if relayListType is Multihop
                if (relayListType is RelayListType.Multihop) {
                    when (relayListType.multihopRelayListType) {
                        MultihopRelayListType.ENTRY -> add(FilterChip.Entry)
                        MultihopRelayListType.EXIT -> add(FilterChip.Exit)
                    }
                }
            }
        }

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
        viewModelScope.launch {
            relayListFilterRepository.updateSelectedOwnership(
                relayListType = relayListType,
                ownership = Constraint.Any,
            )
        }
    }

    fun removeProviderFilter() {
        viewModelScope.launch {
            relayListFilterRepository.updateSelectedProviders(
                relayListType = relayListType,
                providers = Constraint.Any,
            )
        }
    }

    fun setAsEntry(item: RelayItem) {
        viewModelScope.launch {
            modifyMultihop(MultihopChange.Entry(item))
            // If multihop is not turned on, turn it on and show a snackbar to the user
            if (
                wireguardConstraintsRepository.wireguardConstraints.value?.isMultihopEnabled ==
                    false
            ) {
                wireguardConstraintsRepository.setMultihop(true)
                _uiSideEffect.send(SearchLocationSideEffect.MultihopChanged(true))
            }
        }
    }

    fun setAsExit(item: RelayItem) {
        viewModelScope.launch { modifyMultihop(MultihopChange.Exit(item)) }
    }

    fun setMultihop(enable: Boolean) {
        viewModelScope.launch {
            wireguardConstraintsRepository.setMultihop(enable)
            _uiSideEffect.send(SearchLocationSideEffect.MultihopChanged(enable))
        }
    }

    fun onToggleExpand(item: RelayItemId, parent: CustomListId? = null, expand: Boolean) {
        _expandOverrides.onToggleExpandMap(item = item, parent = parent, expand = expand)
    }

    private fun Set<String>.with(overrides: Map<String, Boolean>): Set<String> =
        this + overrides.filterValues { expanded -> expanded }.keys -
            overrides.filterValues { expanded -> !expanded }.keys

    companion object {
        private const val EMPTY_SEARCH_TERM = ""
    }
}

sealed interface SearchLocationSideEffect {
    data class LocationSelected(val relayListType: RelayListType) : SearchLocationSideEffect

    data class CustomListActionToast(val resultData: CustomListActionResultData) :
        SearchLocationSideEffect

    data class MultihopChanged(val enabled: Boolean) : SearchLocationSideEffect

    data class RelayItemInactive(val relayItem: RelayItem) : SearchLocationSideEffect

    data class EntryAlreadySelected(val relayItem: RelayItem) : SearchLocationSideEffect

    data class ExitAlreadySelected(val relayItem: RelayItem) : SearchLocationSideEffect

    data object GenericError : SearchLocationSideEffect
}
