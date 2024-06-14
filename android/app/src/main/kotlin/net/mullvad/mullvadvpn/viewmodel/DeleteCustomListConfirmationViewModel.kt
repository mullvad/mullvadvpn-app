package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.Deleted
import net.mullvad.mullvadvpn.compose.state.DeleteCustomListUiState
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.DeleteWithUndoError

class DeleteCustomListConfirmationViewModel(
    private val customListId: CustomListId,
    private val customListActionUseCase: CustomListActionUseCase
) : ViewModel() {
    private val _uiSideEffect = Channel<DeleteCustomListConfirmationSideEffect>(Channel.BUFFERED)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val _error = MutableStateFlow<DeleteWithUndoError?>(null)

    val uiState =
        _error
            .map { DeleteCustomListUiState(it) }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                DeleteCustomListUiState(null)
            )

    fun deleteCustomList() {
        viewModelScope.launch {
            _error.emit(null)
            customListActionUseCase(CustomListAction.Delete(customListId))
                .fold(
                    { _error.tryEmit(it) },
                    {
                        _uiSideEffect.send(
                            DeleteCustomListConfirmationSideEffect.ReturnWithResult(it)
                        )
                    }
                )
        }
    }
}

sealed interface DeleteCustomListConfirmationSideEffect {
    data class ReturnWithResult(val result: Deleted) : DeleteCustomListConfirmationSideEffect
}
