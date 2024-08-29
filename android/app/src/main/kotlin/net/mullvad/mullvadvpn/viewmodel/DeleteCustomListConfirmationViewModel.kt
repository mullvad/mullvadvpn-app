package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.destinations.DeleteCustomListDestination
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.state.DeleteCustomListUiState
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.DeleteWithUndoError

class DeleteCustomListConfirmationViewModel(
    private val customListActionUseCase: CustomListActionUseCase,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    private val navArgs = DeleteCustomListDestination.argsFrom(savedStateHandle)
    private val name: CustomListName = navArgs.name
    private val customListId: CustomListId = navArgs.customListId

    private val _uiSideEffect = Channel<DeleteCustomListConfirmationSideEffect>(Channel.BUFFERED)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val _error = MutableStateFlow<DeleteWithUndoError?>(null)

    val uiState =
        _error
            .map { DeleteCustomListUiState(name, it) }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                DeleteCustomListUiState(name, null),
            )

    fun deleteCustomList() {
        viewModelScope.launch {
            _error.emit(null)
            customListActionUseCase(CustomListAction.Delete(customListId))
                .fold(
                    { _error.tryEmit(it) },
                    {
                        _uiSideEffect.send(
                            DeleteCustomListConfirmationSideEffect.ReturnWithResult(
                                CustomListActionResultData.Success.Deleted(
                                    customListName = it.name,
                                    undo = it.undo,
                                )
                            )
                        )
                    },
                )
        }
    }
}

sealed interface DeleteCustomListConfirmationSideEffect {
    data class ReturnWithResult(val result: CustomListActionResultData.Success.Deleted) :
        DeleteCustomListConfirmationSideEffect
}
