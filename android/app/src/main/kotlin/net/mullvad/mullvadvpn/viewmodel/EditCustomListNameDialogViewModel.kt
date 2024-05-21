package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.compose.state.EditCustomListNameUiState
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.RenameCustomListError

class EditCustomListNameDialogViewModel(
    private val customListId: net.mullvad.mullvadvpn.lib.model.CustomListId,
    private val initialName: CustomListName,
    private val customListActionUseCase: CustomListActionUseCase
) : ViewModel() {

    private val _uiSideEffect =
        Channel<EditCustomListNameDialogSideEffect>(1, BufferOverflow.DROP_OLDEST)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val _error = MutableStateFlow<RenameCustomListError?>(null)

    val uiState =
        _error
            .map { EditCustomListNameUiState(name = initialName.value, error = it) }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                EditCustomListNameUiState(name = initialName.value)
            )

    fun updateCustomListName(name: String) {
        viewModelScope.launch {
            customListActionUseCase
                .performAction(
                    CustomListAction.Rename(
                        id = customListId,
                        name = initialName,
                        newName = CustomListName.fromString(name)
                    )
                )
                .fold(
                    { _error.emit(it) },
                    { _uiSideEffect.send(EditCustomListNameDialogSideEffect.ReturnWithResult(it)) }
                )
        }
    }

    fun clearError() {
        viewModelScope.launch { _error.emit(null) }
    }
}

sealed interface EditCustomListNameDialogSideEffect {
    data class ReturnWithResult(val result: CustomListResult.Renamed) :
        EditCustomListNameDialogSideEffect
}
