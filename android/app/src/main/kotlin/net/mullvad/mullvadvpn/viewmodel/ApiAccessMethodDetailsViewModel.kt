package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.ApiAccessMethodDetailsUiState
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodType
import net.mullvad.mullvadvpn.repository.ApiAccessRepository
import net.mullvad.mullvadvpn.usecase.TestApiAccessMethodInput
import net.mullvad.mullvadvpn.usecase.TestApiAccessMethodState
import net.mullvad.mullvadvpn.usecase.TestApiAccessMethodUseCase

class ApiAccessMethodDetailsViewModel(
    private val apiAccessMethodId: ApiAccessMethodId,
    private val apiAccessRepository: ApiAccessRepository,
    private val testApiAccessMethodUseCase: TestApiAccessMethodUseCase
) : ViewModel() {
    private val _uiSideEffect = Channel<ApiAccessMethodDetailsSideEffect>(Channel.BUFFERED)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()
    private val testApiAccessMethodState = MutableStateFlow<TestApiAccessMethodState?>(null)
    val uiState =
        combine(
                apiAccessRepository.flowApiAccessMethodById(apiAccessMethodId),
                apiAccessRepository.flowEnabledApiAccessMethods(),
                apiAccessRepository.currentAccessMethod,
                testApiAccessMethodState
            ) {
                apiAccessMethod,
                enabledApiAccessMethods,
                currentAccessMethod,
                testApiAccessMethodState ->
                ApiAccessMethodDetailsUiState.Content(
                    name = apiAccessMethod.name,
                    enabled = apiAccessMethod.enabled,
                    canBeEdited =
                        apiAccessMethod.apiAccessMethodType is ApiAccessMethodType.CustomProxy,
                    canBeDisabled = enabledApiAccessMethods.any { it.id != apiAccessMethodId },
                    currentMethod = currentAccessMethod?.id == apiAccessMethodId,
                    testApiAccessMethodState = testApiAccessMethodState
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                ApiAccessMethodDetailsUiState.Loading
            )

    fun delete() {
        viewModelScope.launch {
            apiAccessRepository
                .removeApiAccessMethod(apiAccessMethodId = apiAccessMethodId)
                .fold(
                    { _uiSideEffect.send(ApiAccessMethodDetailsSideEffect.GenericError) },
                    { _uiSideEffect.send(ApiAccessMethodDetailsSideEffect.CloseScreen) }
                )
        }
    }

    fun setCurrentMethod() {
        viewModelScope.launch {
            apiAccessRepository.setApiAccessMethod(apiAccessMethodId = apiAccessMethodId).onLeft {
                _uiSideEffect.send(ApiAccessMethodDetailsSideEffect.GenericError)
            }
        }
    }

    fun testMethod() {
        viewModelScope.launch {
            testApiAccessMethodUseCase
                .testApiAccessMethod(TestApiAccessMethodInput.TestExistingMethod(apiAccessMethodId))
                .collect(testApiAccessMethodState)
        }
    }

    fun setEnableMethod(enable: Boolean) {
        viewModelScope.launch {
            apiAccessRepository.setEnabledApiAccessMethod(apiAccessMethodId, enable).onLeft {
                _uiSideEffect.send(ApiAccessMethodDetailsSideEffect.GenericError)
            }
        }
    }

    fun openEditPage() {
        viewModelScope.launch {
            _uiSideEffect.send(ApiAccessMethodDetailsSideEffect.OpenEditPage(apiAccessMethodId))
        }
    }
}

sealed interface ApiAccessMethodDetailsSideEffect {
    data class OpenEditPage(val apiAccessMethodId: ApiAccessMethodId) :
        ApiAccessMethodDetailsSideEffect

    data object GenericError : ApiAccessMethodDetailsSideEffect

    data object CloseScreen : ApiAccessMethodDetailsSideEffect
}
