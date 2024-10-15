package net.mullvad.mullvadvpn.viewmodel.location

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import co.touchlab.kermit.Logger
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.state.RelayListSelection
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState.Content
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.usecase.FilterChipUseCase
import net.mullvad.mullvadvpn.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.usecase.SelectedLocationUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.usecase.customlists.FilterCustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.util.combine

@Suppress("TooManyFunctions")
class SelectLocationViewModel(
    private val relayListFilterRepository: RelayListFilterRepository,
    private val filteredCustomListRelayItemsUseCase: FilterCustomListsRelayItemUseCase,
    private val customListsRepository: CustomListsRepository,
    private val customListActionUseCase: CustomListActionUseCase,
    private val filteredRelayListUseCase: FilteredRelayListUseCase,
    private val relayListRepository: RelayListRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val selectedLocationUseCase: SelectedLocationUseCase,
    customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    filterChipUseCase: FilterChipUseCase,
) : ViewModel() {
    private val _relayListSelection: MutableStateFlow<RelayListSelection> =
        MutableStateFlow(initialRelayListSelection())
    private val _expandedItemsEntry: MutableStateFlow<Set<String>> =
        MutableStateFlow(initialExpandEntry())
    private val _expandedItemsExit: MutableStateFlow<Set<String>> =
        MutableStateFlow(initialExpandExit())

    val uiState =
        combine(
                relayListItems(),
                filterChipUseCase(),
                customListsRelayItemUseCase(),
                wireguardConstraintsRepository.wireguardConstraints,
                _relayListSelection,
            ) { relayListItems, filterChips, customLists, wireguardConstraints, relayListSelection
                ->
                Content(
                    filterChips = filterChips,
                    relayListItems = relayListItems,
                    customLists = customLists,
                    multihopEnabled = wireguardConstraints?.useMultihop ?: false,
                    relayListSelection = relayListSelection,
                )
            }
            .stateIn(viewModelScope, SharingStarted.Lazily, SelectLocationUiState.Loading)

    private val _uiSideEffect = Channel<SelectLocationSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private fun initialExpandEntry(): Set<String> = buildSet {
        val selectedEntryLocation =
            wireguardConstraintsRepository.wireguardConstraints.value?.entryLocation
        return initialExpand(selectedEntryLocation?.getOrNull())
    }

    private fun initialExpandExit(): Set<String> {
        val selectedExitLocation = relayListRepository.selectedLocation.value
        return initialExpand(selectedExitLocation.getOrNull())
    }

    private fun initialExpand(item: RelayItemId?): Set<String> = buildSet {
        when (item) {
            is GeoLocationId.City -> {
                Logger.d("GC item: $item")
                add(item.country.code)
            }
            is GeoLocationId.Hostname -> {
                Logger.d("GH item: $item")
                add(item.country.code)
                add(item.city.code)
            }
            is CustomListId,
            is GeoLocationId.Country,
            null -> {
                Logger.d("NO item: $item")
                /* No expands */
            }
        }
    }

    private fun initialRelayListSelection() =
        if (wireguardConstraintsRepository.wireguardConstraints.value?.useMultihop == true) {
            RelayListSelection.Entry
        } else {
            RelayListSelection.Exit
        }

    private fun relayListItems() =
        combine(
            filteredRelayListUseCase(),
            filteredCustomListRelayItemsUseCase(),
            selectedLocationUseCase(),
            _expandedItemsEntry,
            _expandedItemsExit,
            _relayListSelection,
        ) {
            relayCountries,
            customLists,
            selectedItem,
            expandedItemsEntry,
            expandedItemsExit,
            relayListSelection ->
            relayListItems(
                relayCountries = relayCountries,
                customLists = customLists,
                selectedItem = selectedItem.getForRelayListSelect(relayListSelection),
                disabledItem = selectedItem.getForRelayListDisabled(relayListSelection),
                expandedItems =
                    when (relayListSelection) {
                        RelayListSelection.Entry -> expandedItemsEntry
                        RelayListSelection.Exit -> expandedItemsExit
                    },
            )
        }

    fun selectRelayList(relayListSelection: RelayListSelection) {
        viewModelScope.launch { _relayListSelection.emit(relayListSelection) }
    }

    fun selectRelay(relayItem: RelayItem) {
        viewModelScope.launch {
            selectRelayItem(
                    relayItem = relayItem,
                    relayListSelection = _relayListSelection.value,
                    selectEntryLocation = wireguardConstraintsRepository::setEntryLocation,
                    selectExitLocation = relayListRepository::updateSelectedRelayLocation,
                )
                .fold(
                    { _uiSideEffect.send(SelectLocationSideEffect.GenericError) },
                    {
                        when (_relayListSelection.value) {
                            RelayListSelection.Entry ->
                                _relayListSelection.emit(RelayListSelection.Exit)
                            RelayListSelection.Exit ->
                                _uiSideEffect.send(SelectLocationSideEffect.CloseScreen)
                        }
                    },
                )
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
            _uiSideEffect.send(SelectLocationSideEffect.CustomListActionToast(result))
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
            _uiSideEffect.trySend(SelectLocationSideEffect.CustomListActionToast(result))
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
        when (_relayListSelection.value) {
            RelayListSelection.Entry ->
                _expandedItemsEntry.onToggleExpand(item = item, parent = parent, expand = expand)
            RelayListSelection.Exit ->
                _expandedItemsExit.onToggleExpand(item = item, parent = parent, expand = expand)
        }
    }
}

sealed interface SelectLocationSideEffect {
    data object CloseScreen : SelectLocationSideEffect

    data class CustomListActionToast(val resultData: CustomListActionResultData) :
        SelectLocationSideEffect

    data object GenericError : SelectLocationSideEffect
}
