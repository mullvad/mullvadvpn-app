package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import com.ramcosta.composedestinations.generated.destinations.CustomListSheetDestination
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.receiveAsFlow
import net.mullvad.mullvadvpn.compose.state.CustomListSheetUiState

class CustomListSheetViewModel(
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    private val navArgs = CustomListSheetDestination.argsFrom(savedStateHandle)

    private val _uiSideEffect = Channel<CustomListSheetSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState: StateFlow<CustomListSheetUiState> =
        MutableStateFlow(
            CustomListSheetUiState(
                customListId = navArgs.customListId,
                customListName = navArgs.customListName
            )
        )
}

sealed interface CustomListSheetSideEffect {
    data object GenericError : CustomListSheetSideEffect
}
