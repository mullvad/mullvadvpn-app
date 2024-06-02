package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.SaveApiAccessMethodUiState
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodType
import net.mullvad.mullvadvpn.lib.model.NewAccessMethod
import net.mullvad.mullvadvpn.repository.ApiAccessRepository

class SaveApiAccessMethodViewModel(
    private val newAccessMethod: NewAccessMethod,
    private val apiAccessRepository: ApiAccessRepository
) : ViewModel() {
    private val _sideEffects = Channel<SaveApiAccessMethodSideEffect>()
    val sideEffect = _sideEffects.receiveAsFlow()
    private val _uiState =
        MutableStateFlow<SaveApiAccessMethodUiState>(SaveApiAccessMethodUiState.Testing)
    val uiState: StateFlow<SaveApiAccessMethodUiState> = _uiState

    private var testingJob: Job? = null

    init {
        testingJob =
            viewModelScope.launch {
                val customProxy =
                    newAccessMethod.apiAccessMethodType as? ApiAccessMethodType.CustomProxy
                        ?: error("Access method needs to be custom")
                apiAccessRepository
                    .testCustomApiAccessMethod(customProxy)
                    .fold(
                        { _uiState.emit(SaveApiAccessMethodUiState.TestingFailed) },
                        { save(afterSuccessful = true) }
                    )
            }
    }

    fun save(afterSuccessful: Boolean = false) {
        viewModelScope.launch {
            _uiState.emit(
                if (afterSuccessful) {
                    SaveApiAccessMethodUiState.SavingAfterSuccessful
                } else {
                    SaveApiAccessMethodUiState.SavingAfterFailure
                }
            )
            apiAccessRepository
                .addApiAccessMethod(newAccessMethod)
                .fold(
                    {
                        _sideEffects.send(SaveApiAccessMethodSideEffect.CouldNotSaveApiAccessMethod)
                    },
                    {
                        _sideEffects.send(
                            SaveApiAccessMethodSideEffect.SuccessfullyCreatedApiMethod
                        )
                    }
                )
        }
    }

    fun cancel() {
        viewModelScope.launch {
            testingJob?.cancel(message = "Cancelled by user")
            _sideEffects.send(SaveApiAccessMethodSideEffect.Cancel)
        }
    }
}

sealed interface SaveApiAccessMethodSideEffect {
    data object SuccessfullyCreatedApiMethod : SaveApiAccessMethodSideEffect

    data object CouldNotSaveApiAccessMethod : SaveApiAccessMethodSideEffect

    data object Cancel : SaveApiAccessMethodSideEffect
}
