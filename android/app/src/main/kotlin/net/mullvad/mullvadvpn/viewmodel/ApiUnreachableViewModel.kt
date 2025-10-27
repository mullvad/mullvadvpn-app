package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.destinations.ApiUnreachableInfoDestination
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.dialog.info.ApiUnreachableInfoDialogNavArgs
import net.mullvad.mullvadvpn.compose.state.ApiUnreachableUiState
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.ui.component.NEWLINE_STRING
import net.mullvad.mullvadvpn.repository.ApiAccessRepository
import net.mullvad.mullvadvpn.usecase.SupportEmailUseCase

class ApiUnreachableViewModel(
    private val apiAccessRepository: ApiAccessRepository,
    private val supportEmailUseCase: SupportEmailUseCase,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    private val navArgs = ApiUnreachableInfoDestination.argsFrom(savedStateHandle)

    private val noEmailAppAvailable = MutableStateFlow(false)
    private val hasEnabledAllApiAccessMethods = MutableStateFlow(false)

    private val _uiSideEffect = Channel<ApiUnreachableSideEffect>(Channel.BUFFERED)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState =
        combine(noEmailAppAvailable, hasEnabledAllApiAccessMethods) {
                noEmailAppAvailable,
                hasEnabledAllApiAccessMethods ->
                ApiUnreachableUiState(
                    showEnableAllAccessMethodsButton = !hasEnabledAllApiAccessMethods,
                    noEmailAppAvailable = noEmailAppAvailable,
                    loginAction = navArgs.action,
                )
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(stopTimeout = VIEW_MODEL_STOP_TIMEOUT),
                initialValue =
                    ApiUnreachableUiState(
                        showEnableAllAccessMethodsButton = false,
                        noEmailAppAvailable = false,
                        loginAction = navArgs.action,
                    ),
            )

    init {
        viewModelScope.launch {
            hasEnabledAllApiAccessMethods.emit(
                apiAccessRepository.accessMethods.filterNotNull().first().all { it.enabled }
            )
        }
    }

    fun enableAllApiAccess() {
        viewModelScope.launch {
            apiAccessRepository
                .enableAllApiAccessMethods()
                .fold(
                    {
                        _uiSideEffect.send(ApiUnreachableSideEffect.EnableAllApiAccessMethods.Error)
                    },
                    {
                        _uiSideEffect.send(
                            ApiUnreachableSideEffect.EnableAllApiAccessMethods.Success(navArgs)
                        )
                    },
                )
        }
    }

    fun sendProblemReportEmail() {
        viewModelScope.launch {
            noEmailAppAvailable.emit(false)
            val supportEmail = supportEmailUseCase()
            _uiSideEffect.send(
                ApiUnreachableSideEffect.SendEmail(
                    address = supportEmail.address,
                    subject = supportEmail.subject,
                    logs = supportEmail.logs.joinToString(NEWLINE_STRING),
                )
            )
        }
    }

    fun noEmailAppAvailable() {
        viewModelScope.launch { noEmailAppAvailable.emit(true) }
    }
}

sealed interface ApiUnreachableSideEffect {
    data class SendEmail(val address: String, val subject: String, val logs: String) :
        ApiUnreachableSideEffect

    sealed interface EnableAllApiAccessMethods : ApiUnreachableSideEffect {
        data class Success(val navArgs: ApiUnreachableInfoDialogNavArgs) : EnableAllApiAccessMethods

        data object Error : EnableAllApiAccessMethods
    }
}
