package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.destinations.SaveApiAccessMethodDestination
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.SaveApiAccessMethodUiState
import net.mullvad.mullvadvpn.compose.state.TestApiAccessMethodState
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.NewAccessMethodSetting
import net.mullvad.mullvadvpn.repository.ApiAccessRepository

class SaveApiAccessMethodViewModel(
    private val apiAccessRepository: ApiAccessRepository,
    savedStateHandle: SavedStateHandle
) : ViewModel() {
    private val navArgs = SaveApiAccessMethodDestination.argsFrom(savedStateHandle)
    private val apiAccessMethodId: ApiAccessMethodId? = navArgs.id
    private val apiAccessMethodName: ApiAccessMethodName = navArgs.name
    private val customProxy: ApiAccessMethod.CustomProxy = navArgs.customProxy

    private val _uiSideEffect = Channel<SaveApiAccessMethodSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()
    private val _uiState = MutableStateFlow(SaveApiAccessMethodUiState())
    val uiState: StateFlow<SaveApiAccessMethodUiState> = _uiState

    init {
        viewModelScope.launch {
            apiAccessRepository
                .testCustomApiAccessMethod(customProxy)
                .fold(
                    {
                        _uiState.update {
                            it.copy(testingState = TestApiAccessMethodState.Result.Failure)
                        }
                    },
                    {
                        _uiState.update {
                            it.copy(testingState = TestApiAccessMethodState.Result.Successful)
                        }
                        save()
                    }
                )
        }
    }

    fun save() {
        viewModelScope.launch {
            _uiState.update { it.copy(isSaving = true) }
            if (apiAccessMethodId != null) {
                updateAccessMethod(
                    id = apiAccessMethodId,
                    name = apiAccessMethodName,
                    apiAccessMethod = customProxy
                )
            } else {
                addNewAccessMethod(
                    NewAccessMethodSetting(
                        name = apiAccessMethodName,
                        enabled = true,
                        apiAccessMethod = customProxy
                    )
                )
            }
        }
    }

    private suspend fun addNewAccessMethod(newAccessMethodSetting: NewAccessMethodSetting) {
        apiAccessRepository
            .addApiAccessMethod(newAccessMethodSetting)
            .fold(
                { _uiSideEffect.send(SaveApiAccessMethodSideEffect.CouldNotSaveApiAccessMethod) },
                { _uiSideEffect.send(SaveApiAccessMethodSideEffect.SuccessfullyCreatedApiMethod) }
            )
    }

    private suspend fun updateAccessMethod(
        id: ApiAccessMethodId,
        name: ApiAccessMethodName,
        apiAccessMethod: ApiAccessMethod.CustomProxy
    ) {
        apiAccessRepository
            .updateApiAccessMethod(
                apiAccessMethodId = id,
                apiAccessMethodName = name,
                apiAccessMethod = apiAccessMethod
            )
            .fold(
                { _uiSideEffect.send(SaveApiAccessMethodSideEffect.CouldNotSaveApiAccessMethod) },
                { _uiSideEffect.send(SaveApiAccessMethodSideEffect.SuccessfullyCreatedApiMethod) }
            )
    }
}

sealed interface SaveApiAccessMethodSideEffect {
    data object SuccessfullyCreatedApiMethod : SaveApiAccessMethodSideEffect

    data object CouldNotSaveApiAccessMethod : SaveApiAccessMethodSideEffect
}
