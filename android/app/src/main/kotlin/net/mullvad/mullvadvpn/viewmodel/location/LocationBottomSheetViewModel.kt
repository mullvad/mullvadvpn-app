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
                                canBeSetAsEntry =
                                    validate(locationBottomSheetState.item, entrySelectable),
                                canBeSetAsExit =
                                    validate(locationBottomSheetState.item, exitSelectable),
                                canBeRemovedAsEntry =
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
                                canBeSetAsEntry =
                                    validate(locationBottomSheetState.item, entrySelectable),
                                canBeSetAsExit =
                                    validate(locationBottomSheetState.item, exitSelectable),
                                canBeRemovedAsEntry =
                                    selectedLocation.entryLocation()?.getOrNull() ==
                                        locationBottomSheetState.item.id,
                            )
                        )
                    is LocationBottomSheetState.ShowLocationBottomSheet ->
                        Lc.Content(
                            LocationBottomSheetUiState.Location(
                                item = locationBottomSheetState.item,
                                customLists = customLists,
                                canBeSetAsEntry =
                                    validate(locationBottomSheetState.item, entrySelectable),
                                canBeSetAsExit =
                                    validate(locationBottomSheetState.item, exitSelectable),
                                canBeRemovedAsEntry =
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

    private fun validate(relayItem: RelayItem, geoLocationIds: Set<GeoLocationId>): Boolean =
        when (relayItem) {
            is RelayItem.Location -> geoLocationIds.contains(relayItem.id)
            is RelayItem.CustomList ->
                relayItem.locations.withDescendants().any { geoLocationIds.contains(it.id) }
        }
}
