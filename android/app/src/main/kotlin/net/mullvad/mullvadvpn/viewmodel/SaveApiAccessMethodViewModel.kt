package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.SaveApiAccessMethodUiState
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodType
import net.mullvad.mullvadvpn.lib.model.NewAccessMethod
import net.mullvad.mullvadvpn.lib.model.TestApiAccessMethodState
import net.mullvad.mullvadvpn.repository.ApiAccessRepository

class SaveApiAccessMethodViewModel(
    private val apiAccessMethodId: ApiAccessMethodId?,
    private val apiAccessMethodName: ApiAccessMethodName,
    private val enabled: Boolean,
    private val customProxy: ApiAccessMethodType.CustomProxy,
    private val apiAccessRepository: ApiAccessRepository
) : ViewModel() {
    private val _uiSideEffect = Channel<SaveApiAccessMethodSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()
    private val _uiState = MutableStateFlow(SaveApiAccessMethodUiState())
    val uiState: StateFlow<SaveApiAccessMethodUiState> = _uiState

    private var testingJob: Job? = null

    init {
        testingJob =
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
                    ApiAccessMethod(
                        id = apiAccessMethodId,
                        name = apiAccessMethodName,
                        enabled = enabled,
                        apiAccessMethodType = customProxy
                    )
                )
            } else {
                addNewAccessMethod(
                    NewAccessMethod(
                        name = apiAccessMethodName,
                        enabled = enabled,
                        apiAccessMethodType = customProxy
                    )
                )
            }
        }
    }

    fun cancel() {
        viewModelScope.launch {
            testingJob?.cancel(message = "Cancelled by user")
            _uiSideEffect.send(SaveApiAccessMethodSideEffect.Cancel)
        }
    }

    private suspend fun addNewAccessMethod(newAccessMethod: NewAccessMethod) {
        apiAccessRepository
            .addApiAccessMethod(newAccessMethod)
            .fold(
                { _uiSideEffect.send(SaveApiAccessMethodSideEffect.CouldNotSaveApiAccessMethod) },
                { _uiSideEffect.send(SaveApiAccessMethodSideEffect.SuccessfullyCreatedApiMethod) }
            )
    }

    private suspend fun updateAccessMethod(apiAccessMethod: ApiAccessMethod) {
        apiAccessRepository
            .updateApiAccessMethod(apiAccessMethod)
            .fold(
                { _uiSideEffect.send(SaveApiAccessMethodSideEffect.CouldNotSaveApiAccessMethod) },
                { _uiSideEffect.send(SaveApiAccessMethodSideEffect.SuccessfullyCreatedApiMethod) }
            )
    }
}

sealed interface SaveApiAccessMethodSideEffect {
    data object SuccessfullyCreatedApiMethod : SaveApiAccessMethodSideEffect

    data object CouldNotSaveApiAccessMethod : SaveApiAccessMethodSideEffect

    data object Cancel : SaveApiAccessMethodSideEffect
}
