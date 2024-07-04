package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.destinations.DeleteApiAccessMethodConfirmationDestination
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.DeleteApiAccessMethodUiState
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.RemoveApiAccessMethodError
import net.mullvad.mullvadvpn.repository.ApiAccessRepository

class DeleteApiAccessMethodConfirmationViewModel(
    private val apiAccessRepository: ApiAccessRepository,
    savedStateHandle: SavedStateHandle
) : ViewModel() {
    private val apiAccessMethodId: ApiAccessMethodId =
        DeleteApiAccessMethodConfirmationDestination.argsFrom(savedStateHandle).apiAccessMethodId

    private val _uiSideEffect =
        Channel<DeleteApiAccessMethodConfirmationSideEffect>(Channel.BUFFERED)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val _error = MutableStateFlow<RemoveApiAccessMethodError?>(null)

    val uiState =
        _error
            .map { DeleteApiAccessMethodUiState(it) }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                DeleteApiAccessMethodUiState(null)
            )

    fun deleteApiAccessMethod() {
        viewModelScope.launch {
            _error.emit(null)
            apiAccessRepository
                .removeApiAccessMethod(apiAccessMethodId)
                .fold(
                    { _error.tryEmit(it) },
                    { _uiSideEffect.send(DeleteApiAccessMethodConfirmationSideEffect.Deleted) }
                )
        }
    }
}

sealed interface DeleteApiAccessMethodConfirmationSideEffect {
    data object Deleted : DeleteApiAccessMethodConfirmationSideEffect
}
