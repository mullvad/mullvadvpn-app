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
import net.mullvad.mullvadvpn.compose.state.UpdateCustomListUiState
import net.mullvad.mullvadvpn.model.CreateCustomListError
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.ModifyCustomListError
import net.mullvad.mullvadvpn.model.UpdateCustomListError
import net.mullvad.mullvadvpn.model.CustomListName
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase

class EditCustomListNameDialogViewModel(
    private val customListId: CustomListId,
    private val initialName: String,
    private val customListActionUseCase: CustomListActionUseCase
) : ViewModel() {

    private val _uiSideEffect =
        Channel<EditCustomListNameDialogSideEffect>(1, BufferOverflow.DROP_OLDEST)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val _error = MutableStateFlow<ModifyCustomListError?>(null)

    val uiState =
        _error
            .map { UpdateCustomListUiState(name = initialName, error = it) }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                UpdateCustomListUiState(name = initialName)
            )

    fun updateCustomListName(name: String) {
        viewModelScope.launch {
            customListActionUseCase
                .performAction(
                    CustomListAction.Rename(
                        id = customListId,
                        name = CustomListName.fromString(initialName),
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
