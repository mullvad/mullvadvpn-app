package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.raise.either
import com.ramcosta.composedestinations.generated.destinations.CustomListEntrySheetDestination
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.LocationsChanged
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.viewmodel.CustomListEntrySheetSideEffect.GenericError

data class CustomListEntrySheetUiState(
    val locationName: String,
)

sealed interface CustomListEntrySheetSideEffect {
    data object GenericError : CustomListEntrySheetSideEffect

    data class LocationRemovedFromCustomList(val locationsChanged: LocationsChanged) :
        CustomListEntrySheetSideEffect
}

class CustomListEntrySheetViewModel(
    val customListsRepository: CustomListsRepository,
    val customListActionUseCase: CustomListActionUseCase,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    private val navArgs = CustomListEntrySheetDestination.argsFrom(savedStateHandle)

    private val _uiSideEffect = Channel<CustomListEntrySheetSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState: StateFlow<CustomListEntrySheetUiState> =
        MutableStateFlow(CustomListEntrySheetUiState(locationName = navArgs.name))

    fun removeLocationFromList() =
        viewModelScope.launch {
            either {
                    val customList =
                        customListsRepository.getCustomListById(navArgs.customListId).bind()
                    val newLocations = (customList.locations - navArgs.location)
                    customListActionUseCase(
                            CustomListAction.UpdateLocations(customList.id, newLocations))
                        .bind()
                }
                .fold(
                    { _uiSideEffect.send(GenericError) },
                    {
                        _uiSideEffect.send(
                            CustomListEntrySheetSideEffect.LocationRemovedFromCustomList(it))
                    })
        }
}
