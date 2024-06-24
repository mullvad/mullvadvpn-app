package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.destinations.EditCustomListNameDestination
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.Renamed
import net.mullvad.mullvadvpn.compose.state.EditCustomListNameUiState
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.RenameError

class EditCustomListNameDialogViewModel(
    private val customListActionUseCase: CustomListActionUseCase,
    savedStateHandle: SavedStateHandle
) : ViewModel() {

    private val navArgs = EditCustomListNameDestination.argsFrom(savedStateHandle)
    private val inputName = MutableStateFlow(navArgs.initialName.value)

    private val _uiSideEffect =
        Channel<EditCustomListNameDialogSideEffect>(1, BufferOverflow.DROP_OLDEST)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val _error = MutableStateFlow<RenameError?>(null)

    val uiState =
        combine(inputName, _error) { name, error -> EditCustomListNameUiState(name = name, error) }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                EditCustomListNameUiState(name = navArgs.initialName.value)
            )

    fun updateCustomListName(name: String) {
        viewModelScope.launch {
            customListActionUseCase(
                    CustomListAction.Rename(
                        id = navArgs.customListId,
                        name = navArgs.initialName,
                        newName = CustomListName.fromString(name)
                    )
                )
                .fold(
                    { _error.emit(it) },
                    { _uiSideEffect.send(EditCustomListNameDialogSideEffect.ReturnWithResult(it)) }
                )
        }
    }

    fun onNameChanged(name: String) {
        inputName.value = name
        viewModelScope.launch { _error.emit(null) }
    }
}

sealed interface EditCustomListNameDialogSideEffect {
    data class ReturnWithResult(val result: Renamed) : EditCustomListNameDialogSideEffect
}
