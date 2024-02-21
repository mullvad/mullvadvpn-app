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
import net.mullvad.mullvadvpn.compose.state.CreateCustomListUiState
import net.mullvad.mullvadvpn.model.CreateCustomListResult
import net.mullvad.mullvadvpn.model.CustomListsError
import net.mullvad.mullvadvpn.repository.CustomListsRepository

class CreateCustomListDialogViewModel(
    private val customListsRepository: CustomListsRepository,
) : ViewModel() {

    private val _uiSideEffect =
        Channel<CreateCustomListDialogSideEffect>(1, BufferOverflow.DROP_OLDEST)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val _error = MutableStateFlow<CustomListsError?>(null)

    val uiState =
        _error
            .map { CreateCustomListUiState(it) }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), CreateCustomListUiState())

    fun createCustomList(name: String) {
        viewModelScope.launch {
            when (val result = customListsRepository.createCustomList(name)) {
                is CreateCustomListResult.Ok -> {
                    _uiSideEffect.send(
                        CreateCustomListDialogSideEffect.NavigateToCustomListScreen(result.id)
                    )
                }
                is CreateCustomListResult.Error -> {
                    _error.emit(result.error)
                }
            }
        }
    }

    fun clearError() {
        viewModelScope.launch { _error.emit(null) }
    }
}

sealed interface CreateCustomListDialogSideEffect {

    data class NavigateToCustomListScreen(val customListId: String) :
        CreateCustomListDialogSideEffect
}
