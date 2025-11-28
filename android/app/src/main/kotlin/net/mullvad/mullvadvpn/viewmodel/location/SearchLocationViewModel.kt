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
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.state.MultihopRelayListType
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.compose.state.SearchLocationUiState
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.relaylist.newFilterOnSearch
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.usecase.FilterChip
import net.mullvad.mullvadvpn.usecase.FilterChipUseCase
import net.mullvad.mullvadvpn.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.usecase.ModifyMultihopError
import net.mullvad.mullvadvpn.usecase.ModifyMultihopUseCase
import net.mullvad.mullvadvpn.usecase.MultihopChange
import net.mullvad.mullvadvpn.usecase.SelectRelayItemError
import net.mullvad.mullvadvpn.usecase.SelectSinglehopUseCase
import net.mullvad.mullvadvpn.usecase.SelectedLocationUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.usecase.customlists.FilterCustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.util.Lce
import net.mullvad.mullvadvpn.util.combine

@Suppress("LongParameterList", "TooManyFunctions")
class SearchLocationViewModel(
    private val customListActionUseCase: CustomListActionUseCase,
    private val customListsRepository: CustomListsRepository,
    private val relayListFilterRepository: RelayListFilterRepository,
    private val filterChipUseCase: FilterChipUseCase,
    private val selectSinglehopUseCase: SelectSinglehopUseCase,
    private val modifyMultihopUseCase: ModifyMultihopUseCase,
    private val settingsRepository: SettingsRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val relayListRepository: RelayListRepository,
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
                RelayListType.Single -> selectSinglehop(item = relayItem)
            }
        }
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
                { _uiSideEffect.send(it.toSideEffect(change = change)) },
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
        filterChipUseCase(relayListType).map { filterChips ->
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
        viewModelScope.launch { relayListFilterRepository.updateSelectedOwnership(Constraint.Any) }
    }

    fun removeProviderFilter() {
        viewModelScope.launch { relayListFilterRepository.updateSelectedProviders(Constraint.Any) }
    }

    fun onToggleExpand(item: RelayItemId, parent: CustomListId? = null, expand: Boolean) {
        _expandOverrides.onToggleExpandMap(item = item, parent = parent, expand = expand)
    }

    fun onModifyMultihopError(
        modifyMultihopError: ModifyMultihopError,
        multihopChange: MultihopChange,
    ) {
        viewModelScope.launch {
            _uiSideEffect.send(modifyMultihopError.toSideEffect(multihopChange))
        }
    }

    fun onSelectRelayItemError(selectRelayItemError: SelectRelayItemError) {
        viewModelScope.launch { _uiSideEffect.send(selectRelayItemError.toSideEffect()) }
    }

    fun onMultihopChanged(undoChangeMultihopAction: UndoChangeMultihopAction) {
        viewModelScope.launch {
            _uiSideEffect.send(SearchLocationSideEffect.MultihopChanged(undoChangeMultihopAction))
        }
    }

    fun undoMultihopAction(undoChangeMultihopAction: UndoChangeMultihopAction) {
        viewModelScope.launch {
            when (undoChangeMultihopAction) {
                UndoChangeMultihopAction.Enable ->
                    wireguardConstraintsRepository.setMultihop(true).onLeft {
                        _uiSideEffect.send(SearchLocationSideEffect.GenericError)
                    }
                UndoChangeMultihopAction.Disable ->
                    wireguardConstraintsRepository.setMultihop(false).onLeft {
                        _uiSideEffect.send(SearchLocationSideEffect.GenericError)
                    }
                is UndoChangeMultihopAction.DisableAndSetEntry ->
                    wireguardConstraintsRepository
                        .setMultihopAndEntryLocation(false, undoChangeMultihopAction.relayItemId)
                        .onLeft { _uiSideEffect.send(SearchLocationSideEffect.GenericError) }
                is UndoChangeMultihopAction.DisableAndSetExit ->
                    relayListRepository
                        .updateExitRelayLocationMultihop(
                            false,
                            undoChangeMultihopAction.relayItemId,
                        )
                        .onLeft { _uiSideEffect.send(SearchLocationSideEffect.GenericError) }
            }
        }
    }

    private fun Set<String>.with(overrides: Map<String, Boolean>): Set<String> =
        this + overrides.filterValues { expanded -> expanded }.keys -
            overrides.filterValues { expanded -> !expanded }.keys

    private fun ModifyMultihopError.toSideEffect(change: MultihopChange) =
        when (this) {
            is ModifyMultihopError.EntrySameAsExit ->
                when (change) {
                    is MultihopChange.Entry ->
                        SearchLocationSideEffect.ExitAlreadySelected(relayItem = change.item)
                    is MultihopChange.Exit ->
                        SearchLocationSideEffect.EntryAlreadySelected(relayItem = change.item)
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

sealed interface SearchLocationSideEffect {
    data class LocationSelected(val relayListType: RelayListType) : SearchLocationSideEffect

    data class CustomListActionToast(val resultData: CustomListActionResultData) :
        SearchLocationSideEffect

    data class MultihopChanged(val undoChangeMultihopAction: UndoChangeMultihopAction) :
        SearchLocationSideEffect

    data class RelayItemInactive(val relayItem: RelayItem) : SearchLocationSideEffect

    data class EntryAlreadySelected(val relayItem: RelayItem) : SearchLocationSideEffect

    data class ExitAlreadySelected(val relayItem: RelayItem) : SearchLocationSideEffect

    data object GenericError : SearchLocationSideEffect
}
