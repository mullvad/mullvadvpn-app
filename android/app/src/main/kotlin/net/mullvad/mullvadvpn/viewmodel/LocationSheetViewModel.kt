package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.destinations.LocationSheetDestination
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.SharingStarted.Companion.WhileSubscribed
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.LocationsChanged
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.relaylist.descendants
import net.mullvad.mullvadvpn.relaylist.findByGeoLocationId
import net.mullvad.mullvadvpn.relaylist.withDescendants
import net.mullvad.mullvadvpn.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase

sealed interface LocationUiState {
    val name: String

    data class Loading(override val name: String) : LocationUiState

    data class Content(val location: RelayItem.Location, val customLists: List<CustomListEntry>) :
        LocationUiState {
        override val name: String = location.name
    }
}

data class CustomListEntry(val customList: RelayItem.CustomList, val canAdd: Boolean)

sealed interface LocationSideEffect {
    data object GenericError : LocationSideEffect

    data class LocationAddedToCustomList(val locationsChanged: LocationsChanged) :
        LocationSideEffect
}

class LocationSheetViewModel(
    filteredRelayListUseCase: FilteredRelayListUseCase,
    val customListActionUseCase: CustomListActionUseCase,
    customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    private val navArgs = LocationSheetDestination.argsFrom(savedStateHandle)
    private val geoLocationId = navArgs.id
    private val locationName = navArgs.locationName

    private val _uiSideEffect = Channel<LocationSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState: StateFlow<LocationUiState> =
        combine(
                customListsRelayItemUseCase(),
                filteredRelayListUseCase(),
            ) { customListsRelayItem, relayList ->
                LocationUiState.Content(
                    relayList.findByGeoLocationId(geoLocationId)!!,
                    customListsRelayItem.map {
                        CustomListEntry(
                            it,
                            it.locations.withDescendants().none { it.id == geoLocationId }
                        )
                    }
                )
            }
            .stateIn(viewModelScope, WhileSubscribed(), LocationUiState.Loading(locationName))

    fun addLocationToList(item: RelayItem.Location, customList: RelayItem.CustomList) {
        viewModelScope.launch {
            val newLocations =
                (customList.locations + item).filter { it !in item.descendants() }.map { it.id }
            customListActionUseCase(CustomListAction.UpdateLocations(customList.id, newLocations))
                .fold(
                    { _uiSideEffect.send(LocationSideEffect.GenericError) },
                    { _uiSideEffect.send(LocationSideEffect.LocationAddedToCustomList(it)) },
                )
        }
    }
}
