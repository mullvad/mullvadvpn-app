package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.ApiUnreachableUiState
import net.mullvad.mullvadvpn.lib.ui.component.NEWLINE_STRING
import net.mullvad.mullvadvpn.repository.ApiAccessRepository
import net.mullvad.mullvadvpn.usecase.SupportEmailUseCase

class ApiUnreachableViewModel(
    private val apiAccessRepository: ApiAccessRepository,
    private val supportEmailUseCase: SupportEmailUseCase,
) : ViewModel() {

    private val _uiSideEffect = Channel<ApiUnreachableSideEffect>(Channel.BUFFERED)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val _uiState = MutableStateFlow(ApiUnreachableUiState(false))
    val uiState: StateFlow<ApiUnreachableUiState> = _uiState

    init {
        viewModelScope.launch {
            val hasNotEnabledApiAccessMethod =
                apiAccessRepository.accessMethods.filterNotNull().first().any { !it.enabled }
            _uiState.emit(ApiUnreachableUiState(hasNotEnabledApiAccessMethod))
        }
    }

    fun enableAllApiAccess() {
        viewModelScope.launch { apiAccessRepository.enableAllApiAccessMethods() }
    }

    fun sendProblemReportEmail() {
        viewModelScope.launch {
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
}

sealed interface ApiUnreachableSideEffect {
    data class SendEmail(val address: String, val subject: String, val logs: String) :
        ApiUnreachableSideEffect
}
