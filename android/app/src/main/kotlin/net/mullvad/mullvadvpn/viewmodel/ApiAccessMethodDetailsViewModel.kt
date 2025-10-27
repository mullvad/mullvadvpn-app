package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.Either
import arrow.core.raise.either
import com.ramcosta.composedestinations.generated.destinations.ApiAccessMethodDetailsDestination
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.ApiAccessMethodDetailsUiState
import net.mullvad.mullvadvpn.constant.MINIMUM_LOADING_TIME_MILLIS
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.TestApiAccessMethodError
import net.mullvad.mullvadvpn.repository.ApiAccessRepository
import net.mullvad.mullvadvpn.util.delayAtLeast

class ApiAccessMethodDetailsViewModel(
    private val apiAccessRepository: ApiAccessRepository,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    private var testingJob: Job? = null

    private val apiAccessMethodId: ApiAccessMethodId =
        ApiAccessMethodDetailsDestination.argsFrom(savedStateHandle).accessMethodId

    private val _uiSideEffect = Channel<ApiAccessMethodDetailsSideEffect>(Channel.BUFFERED)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()
    private val isTestingApiAccessMethodState = MutableStateFlow(false)
    val uiState =
        combine(
                apiAccessRepository.apiAccessMethodSettingById(apiAccessMethodId),
                apiAccessRepository.enabledApiAccessMethods(),
                apiAccessRepository.currentAccessMethod,
                isTestingApiAccessMethodState,
            ) {
                apiAccessMethodSetting,
                enabledApiAccessMethods,
                currentAccessMethod,
                isTestingApiAccessMethod ->
                ApiAccessMethodDetailsUiState.Content(
                    apiAccessMethodSetting = apiAccessMethodSetting,
                    isDisableable = enabledApiAccessMethods.any { it.id != apiAccessMethodId },
                    isCurrentMethod = currentAccessMethod?.id == apiAccessMethodId,
                    isTestingAccessMethod = isTestingApiAccessMethod,
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                ApiAccessMethodDetailsUiState.Loading(apiAccessMethodId = apiAccessMethodId),
            )

    fun setCurrentMethod() {
        testingJob =
            viewModelScope.launch {
                either {
                        testMethodById().bind()
                        apiAccessRepository
                            .setCurrentApiAccessMethod(apiAccessMethodId = apiAccessMethodId)
                            .bind()
                    }
                    .onLeft {
                        _uiSideEffect.send(
                            ApiAccessMethodDetailsSideEffect.UnableToSetCurrentMethod(
                                testMethodFailed = it is TestApiAccessMethodError
                            )
                        )
                    }
            }
    }

    fun testMethod() {
        testingJob =
            viewModelScope.launch {
                val result = testMethodById()
                _uiSideEffect.send(
                    ApiAccessMethodDetailsSideEffect.TestApiAccessMethodResult(result.isRight())
                )
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

    fun cancelTestMethod() {
        if (testingJob?.isActive == true) {
            testingJob?.cancel("User cancelled job")
            isTestingApiAccessMethodState.value = false
        }
    }

    private suspend fun testMethodById(): Either<TestApiAccessMethodError, Unit> {
        isTestingApiAccessMethodState.value = true
        return delayAtLeast(MINIMUM_LOADING_TIME_MILLIS) {
                apiAccessRepository.testApiAccessMethodById(apiAccessMethodId)
            }
            .onLeft { isTestingApiAccessMethodState.value = false }
            .onRight { isTestingApiAccessMethodState.value = false }
    }
}

sealed interface ApiAccessMethodDetailsSideEffect {
    data class OpenEditPage(val apiAccessMethodId: ApiAccessMethodId) :
        ApiAccessMethodDetailsSideEffect

    data object GenericError : ApiAccessMethodDetailsSideEffect

    data class TestApiAccessMethodResult(val successful: Boolean) : ApiAccessMethodDetailsSideEffect

    data class UnableToSetCurrentMethod(val testMethodFailed: Boolean) :
        ApiAccessMethodDetailsSideEffect
}
