package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.Either
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
import net.mullvad.mullvadvpn.lib.model.TestApiAccessMethodError
import net.mullvad.mullvadvpn.repository.ApiAccessRepository

class ApiAccessMethodDetailsViewModel(
    private val apiAccessMethodId: ApiAccessMethodId,
    private val apiAccessRepository: ApiAccessRepository
) : ViewModel() {
    private val _uiSideEffect = Channel<ApiAccessMethodDetailsSideEffect>(Channel.BUFFERED)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()
    private val isTestingApiAccessMethodState = MutableStateFlow(false)
    val uiState =
        combine(
                apiAccessRepository.apiAccessMethodById(apiAccessMethodId),
                apiAccessRepository.enabledApiAccessMethods(),
                apiAccessRepository.currentAccessMethod,
                isTestingApiAccessMethodState
            ) {
                apiAccessMethod,
                enabledApiAccessMethods,
                currentAccessMethod,
                isTestingApiAccessMethod ->
                ApiAccessMethodDetailsUiState.Content(
                    apiAccessMethodId = apiAccessMethodId,
                    name = apiAccessMethod.name,
                    enabled = apiAccessMethod.enabled,
                    isEditable =
                        apiAccessMethod.apiAccessMethodType is ApiAccessMethodType.CustomProxy,
                    isDisableable = enabledApiAccessMethods.any { it.id != apiAccessMethodId },
                    isCurrentMethod = currentAccessMethod?.id == apiAccessMethodId,
                    isTestingAccessMethod = isTestingApiAccessMethod
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                ApiAccessMethodDetailsUiState.Loading(apiAccessMethodId = apiAccessMethodId)
            )

    fun setCurrentMethod() {
        viewModelScope.launch {
            testMethodWithStatus().onRight {
                apiAccessRepository
                    .setApiAccessMethod(apiAccessMethodId = apiAccessMethodId)
                    .onLeft { _uiSideEffect.send(ApiAccessMethodDetailsSideEffect.GenericError) }
            }
        }
    }

    fun testMethod() {
        viewModelScope.launch { testMethodWithStatus() }
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

    private suspend fun testMethodWithStatus(): Either<TestApiAccessMethodError, Unit> {
        isTestingApiAccessMethodState.value = true
        return apiAccessRepository
            .testApiAccessMethodById(apiAccessMethodId)
            .onLeft {
                isTestingApiAccessMethodState.value = false
                _uiSideEffect.send(
                    ApiAccessMethodDetailsSideEffect.TestApiAccessMethodResult(false)
                )
            }
            .onRight {
                isTestingApiAccessMethodState.value = false
                ApiAccessMethodDetailsSideEffect.TestApiAccessMethodResult(true)
            }
    }
}

sealed interface ApiAccessMethodDetailsSideEffect {
    data class OpenEditPage(val apiAccessMethodId: ApiAccessMethodId) :
        ApiAccessMethodDetailsSideEffect

    data object GenericError : ApiAccessMethodDetailsSideEffect

    data class TestApiAccessMethodResult(val successful: Boolean) :
        ApiAccessMethodDetailsSideEffect
}
