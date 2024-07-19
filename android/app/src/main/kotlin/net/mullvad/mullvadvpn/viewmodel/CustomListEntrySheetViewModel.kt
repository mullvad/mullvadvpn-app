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
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResult
import net.mullvad.mullvadvpn.compose.communication.GenericError
import net.mullvad.mullvadvpn.compose.state.CustomListEntrySheetUiState
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase

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
            val result =
                either {
                        val customList =
                            customListsRepository.getCustomListById(navArgs.customListId).bind()
                        val newLocations = (customList.locations - navArgs.location)
                        customListActionUseCase(
                                CustomListAction.UpdateLocations(customList.id, newLocations)
                            )
                            .bind()
                    }
                    .fold({ GenericError }, { it })
            _uiSideEffect.send(CustomListEntrySheetSideEffect.LocationRemovedResult(result))
        }
}

sealed interface CustomListEntrySheetSideEffect {
    data class LocationRemovedResult(val result: CustomListActionResult) :
        CustomListEntrySheetSideEffect
}
