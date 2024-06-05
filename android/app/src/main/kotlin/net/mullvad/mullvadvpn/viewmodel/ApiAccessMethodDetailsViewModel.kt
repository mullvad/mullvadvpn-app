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
import net.mullvad.mullvadvpn.lib.model.TestApiAccessMethodState
import net.mullvad.mullvadvpn.repository.ApiAccessRepository
import net.mullvad.mullvadvpn.usecase.TestApiAccessMethodInput
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
                apiAccessRepository.apiAccessMethodById(apiAccessMethodId),
                apiAccessRepository.enabledApiAccessMethods(),
                apiAccessRepository.currentAccessMethod,
                testApiAccessMethodState
            ) {
                apiAccessMethod,
                enabledApiAccessMethods,
                currentAccessMethod,
                testApiAccessMethodState ->
                ApiAccessMethodDetailsUiState.Content(
                    apiAccessMethodId = apiAccessMethodId,
                    name = apiAccessMethod.name,
                    enabled = apiAccessMethod.enabled,
                    isEditable =
                        apiAccessMethod.apiAccessMethodType is ApiAccessMethodType.CustomProxy,
                    isDisableable = enabledApiAccessMethods.any { it.id != apiAccessMethodId },
                    isCurrentMethod = currentAccessMethod?.id == apiAccessMethodId,
                    testApiAccessMethodState = testApiAccessMethodState
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                ApiAccessMethodDetailsUiState.Loading(apiAccessMethodId = apiAccessMethodId)
            )

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
}
