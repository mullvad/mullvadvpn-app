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
import net.mullvad.mullvadvpn.compose.state.UpdateCustomListUiState
import net.mullvad.mullvadvpn.model.CustomListsError
import net.mullvad.mullvadvpn.model.UpdateCustomListResult
import net.mullvad.mullvadvpn.repository.CustomListsRepository

class EditCustomListNameDialogViewModel(
    private val id: String,
    private val customListsRepository: CustomListsRepository
) : ViewModel() {

    private val _uiSideEffect =
        Channel<EditCustomListNameDialogSideEffect>(1, BufferOverflow.DROP_OLDEST)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val _error = MutableStateFlow<CustomListsError?>(null)

    val uiState =
        _error
            .map { UpdateCustomListUiState(it) }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), UpdateCustomListUiState())

    fun updateCustomListName(name: String) {
        viewModelScope.launch {
            when (val result = customListsRepository.updateCustomListName(id, name)) {
                UpdateCustomListResult.Ok -> {
                    _uiSideEffect.send(EditCustomListNameDialogSideEffect.CloseScreen)
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
    data object CloseScreen : EditCustomListNameDialogSideEffect
}
