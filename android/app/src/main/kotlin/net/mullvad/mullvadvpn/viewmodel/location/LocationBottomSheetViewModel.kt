package net.mullvad.mullvadvpn.viewmodel.location

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
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
import net.mullvad.mullvadvpn.usecase.RelayItemCanBeSelectedUseCase
import net.mullvad.mullvadvpn.usecase.SelectedLocationUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.util.Lc

class LocationBottomSheetViewModel(
    private val locationBottomSheetState: LocationBottomSheetState,
    private val relayListType: RelayListType,
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

    fun validate(relayItem: RelayItem, geoLocationIds: Set<GeoLocationId>): SetAsState =
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
