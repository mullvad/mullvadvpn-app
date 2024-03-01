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
import net.mullvad.mullvadvpn.model.UpdateCustomListResult
import net.mullvad.mullvadvpn.repository.CustomListsRepository

class EditCustomListNameDialogViewModel(
    private val action: CustomListAction.Rename,
    private val customListsRepository: CustomListsRepository
) : ViewModel() {

    private val _uiSideEffect =
        Channel<EditCustomListNameDialogSideEffect>(1, BufferOverflow.DROP_OLDEST)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val _error = MutableStateFlow<CustomListsError?>(null)

    val uiState =
        _error
            .map { UpdateCustomListUiState(name = action.name, error = it) }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                UpdateCustomListUiState(name = action.name)
            )

    fun updateCustomListName(name: String) {
        viewModelScope.launch {
            when (
                val result =
                    customListsRepository.updateCustomListName(
                        id = action.customListId,
                        name = name
                    )
            ) {
                UpdateCustomListResult.Ok -> {
                    _uiSideEffect.send(
                        EditCustomListNameDialogSideEffect.ReturnResult(
                            result =
                                CustomListResult.ListRenamed(
                                    name = name,
                                    reverseAction = action.not()
                                )
                        )
                    )
                }
                is UpdateCustomListResult.Error -> {
                    _error.emit(result.error)
                }
            }
        }
    }

    fun clearError() {
        viewModelScope.launch { _error.emit(null) }
    }
}

sealed interface EditCustomListNameDialogSideEffect {
    data class ReturnResult(val result: CustomListResult.ListRenamed) :
        EditCustomListNameDialogSideEffect
}
