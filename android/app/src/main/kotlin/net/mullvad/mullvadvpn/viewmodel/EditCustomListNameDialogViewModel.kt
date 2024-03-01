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
import net.mullvad.mullvadvpn.model.CustomListsError
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsException

class EditCustomListNameDialogViewModel(
    private val customListId: String,
    private val initialName: String,
    private val customListActionUseCase: CustomListActionUseCase
) : ViewModel() {

    private val _uiSideEffect =
        Channel<EditCustomListNameDialogSideEffect>(1, BufferOverflow.DROP_OLDEST)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val _error = MutableStateFlow<CustomListsError?>(null)

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
                        customListId = customListId,
                        name = initialName,
                        newName = name
                    )
                )
                .fold(
                    onSuccess = { result ->
                        _uiSideEffect.send(EditCustomListNameDialogSideEffect.ReturnResult(result))
                    },
                    onFailure = { exception ->
                        if (exception is CustomListsException) {
                            _error.emit(exception.error)
                        } else {
                            _error.emit(CustomListsError.OtherError)
                        }
                    }
                )
        }
    }

    fun clearError() {
        viewModelScope.launch { _error.emit(null) }
    }
}

sealed interface EditCustomListNameDialogSideEffect {
    data class ReturnResult(val result: CustomListResult.Renamed) :
        EditCustomListNameDialogSideEffect
}
