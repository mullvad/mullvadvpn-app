package net.mullvad.mullvadvpn.viewmodel.location

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import co.touchlab.kermit.Logger
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.screen.location.LocationBottomSheetState
import net.mullvad.mullvadvpn.compose.state.LocationBottomSheetUiState
import net.mullvad.mullvadvpn.compose.state.MultihopRelayListType
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.compose.state.SetAsState
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.relaylist.withDescendants
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.usecase.HopSelectionUseCase
import net.mullvad.mullvadvpn.usecase.ModifyMultihopError
import net.mullvad.mullvadvpn.usecase.ModifyMultihopUseCase
import net.mullvad.mullvadvpn.usecase.MultihopChange
import net.mullvad.mullvadvpn.usecase.RelayItemCanBeSelectedUseCase
import net.mullvad.mullvadvpn.usecase.SelectMultiHopUseCase
import net.mullvad.mullvadvpn.usecase.SelectRelayItemError
import net.mullvad.mullvadvpn.usecase.SelectedLocationUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.util.Lc

class LocationBottomSheetViewModel(
    private val locationBottomSheetState: LocationBottomSheetState,
    private val relayListType: RelayListType,
    private val hopSelectionUseCase: HopSelectionUseCase,
    private val modifyMultihopUseCase: ModifyMultihopUseCase,
    private val selectMultiHopUseCase: SelectMultiHopUseCase,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    canBeSelectedUseCase: RelayItemCanBeSelectedUseCase,
    customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    selectedLocationUseCase: SelectedLocationUseCase,
) : ViewModel() {

    val uiState: StateFlow<Lc<Unit, LocationBottomSheetUiState>> =
        combine(canBeSelectedUseCase(), customListsRelayItemUseCase(), selectedLocationUseCase()) {
                (entrySelectable, exitSelectable),
                customLists,
                selectedLocation ->
                when (locationBottomSheetState) {
                    is LocationBottomSheetState.ShowCustomListsEntryBottomSheet ->
                        Lc.Content(
                            LocationBottomSheetUiState.CustomListsEntry(
                                item = locationBottomSheetState.item,
                                setAsEntryState =
                                    setAsEntryState(
                                        relayItem = locationBottomSheetState.item,
                                        geoLocationIds = entrySelectable,
                                        relayListType = relayListType,
                                    ),
                                setAsExitState =
                                    setAsExitState(
                                        relayItem = locationBottomSheetState.item,
                                        geoLocationIds = exitSelectable,
                                        relayListType = relayListType,
                                    ),
                                canDisableMultihop =
                                    false, // Custom list entries are never considered to be
                                // selected
                                customListId = locationBottomSheetState.customListId,
                                customListName =
                                    CustomListName.fromString(
                                        customLists
                                            .firstOrNull {
                                                it.id == locationBottomSheetState.customListId
                                            }
                                            ?.name ?: ""
                                    ),
                            )
                        )
                    is LocationBottomSheetState.ShowEditCustomListBottomSheet ->
                        Lc.Content(
                            LocationBottomSheetUiState.CustomList(
                                item = locationBottomSheetState.item,
                                setAsEntryState =
                                    setAsEntryState(
                                        relayItem = locationBottomSheetState.item,
                                        geoLocationIds = entrySelectable,
                                        relayListType = relayListType,
                                    ),
                                setAsExitState =
                                    setAsExitState(
                                        relayItem = locationBottomSheetState.item,
                                        geoLocationIds = exitSelectable,
                                        relayListType = relayListType,
                                    ),
                                canDisableMultihop =
                                    selectedLocation.entryLocation()?.getOrNull() ==
                                        locationBottomSheetState.item.id,
                            )
                        )
                    is LocationBottomSheetState.ShowLocationBottomSheet ->
                        Lc.Content(
                            LocationBottomSheetUiState.Location(
                                item = locationBottomSheetState.item,
                                customLists = customLists,
                                setAsEntryState =
                                    setAsEntryState(
                                        relayItem = locationBottomSheetState.item,
                                        geoLocationIds = entrySelectable,
                                        relayListType = relayListType,
                                    ),
                                setAsExitState =
                                    setAsExitState(
                                        relayItem = locationBottomSheetState.item,
                                        geoLocationIds = exitSelectable,
                                        relayListType = relayListType,
                                    ),
                                canDisableMultihop =
                                    selectedLocation.entryLocation()?.getOrNull() ==
                                        locationBottomSheetState.item.id,
                            )
                        )
                }
            }
            .stateIn(
                viewModelScope,
                started = SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(Unit),
            )

    fun setAsEntry(
        item: RelayItem,
        onError: (ModifyMultihopError, MultihopChange) -> Unit,
        onUpdateMultihop: (Boolean, MultihopChange?) -> Unit,
    ) {
        viewModelScope.launch {
            val previousEntry = hopSelectionUseCase().first().entry()?.getOrNull()
            val change = MultihopChange.Entry(item)
            val isMultihopEnabled = isMultihopEnabled()
            if (isMultihopEnabled) {
                    modifyMultihopUseCase(change = change)
                } else {
                    modifyMultihopUseCase(change = change, dryRun = true).map {
                        wireguardConstraintsRepository.setMultihopAndEntryLocation(
                            multihopEnabled = true,
                            entryRelayItemId = item.id,
                        )
                    }
                }
                .fold(
                    { onError(it, change) },
                    {
                        if (!isMultihopEnabled) {
                            onUpdateMultihop(
                                true,
                                previousEntry?.let { MultihopChange.Entry(previousEntry) },
                            )
                        }
                    },
                )
        }
    }

    fun setAsExit(
        item: RelayItem,
        onModifyMultihopError: (ModifyMultihopError, MultihopChange) -> Unit,
        onRelayItemError: (SelectRelayItemError) -> Unit,
        onUpdateMultihop: (Boolean, MultihopChange?) -> Unit,
    ) {
        viewModelScope.launch {
            val previousExit = hopSelectionUseCase().first().exit()?.getOrNull()
            val isMultihopEnabled = isMultihopEnabled()
            if (isMultihopEnabled) {
                    modifyMultihopUseCase(MultihopChange.Exit(item = item))
                } else {
                    // If we are in singlehop mode we want to set a new multihop were the previous
                    // exit
                    // is set as an entry, and the new exit is set as exit
                    // After that we turn on multihop
                    selectMultiHopUseCase(entry = previousExit, exit = item)
                }
                .fold(
                    { error ->
                        when (error) {
                            is ModifyMultihopError ->
                                onModifyMultihopError(error, MultihopChange.Exit(item))
                            is SelectRelayItemError -> onRelayItemError(error)
                            else -> error("Error not supported")
                        }
                    },
                    {
                        onUpdateMultihop(
                            true,
                            previousExit?.let { MultihopChange.Exit(previousExit) },
                        )
                    },
                )
        }
    }

    fun disableMultihop(onUpdateMultihop: (Boolean, MultihopChange?) -> Unit) {
        viewModelScope.launch {
            wireguardConstraintsRepository
                .setMultihop(false)
                .fold({ Logger.e("Set multihop error") }, { onUpdateMultihop(false, null) })
        }
    }

    private fun isMultihopEnabled() =
        wireguardConstraintsRepository.wireguardConstraints.value?.isMultihopEnabled ?: false

    private fun RelayListType.isMultihopEntry() =
        this is RelayListType.Multihop && this.multihopRelayListType == MultihopRelayListType.ENTRY

    private fun RelayListType.isMultihopExit() =
        this is RelayListType.Multihop && this.multihopRelayListType == MultihopRelayListType.EXIT

    private fun setAsEntryState(
        relayItem: RelayItem,
        geoLocationIds: Set<GeoLocationId>,
        relayListType: RelayListType,
    ) =
        if (relayListType.isMultihopEntry()) {
            SetAsState.HIDDEN
        } else {
            validate(relayItem, geoLocationIds)
        }

    private fun setAsExitState(
        relayItem: RelayItem,
        geoLocationIds: Set<GeoLocationId>,
        relayListType: RelayListType,
    ) =
        if (relayListType.isMultihopExit()) {
            SetAsState.HIDDEN
        } else {
            validate(relayItem, geoLocationIds)
        }

    private fun validate(relayItem: RelayItem, geoLocationIds: Set<GeoLocationId>): SetAsState =
        if (
            when (relayItem) {
                is RelayItem.Location -> geoLocationIds.contains(relayItem.id)
                is RelayItem.CustomList ->
                    relayItem.locations.withDescendants().any { geoLocationIds.contains(it.id) }
            }
        ) {
            SetAsState.ENABLED
        } else {
            SetAsState.DISABLED
        }
}
