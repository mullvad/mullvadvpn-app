package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.async
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.constant.MINIMUM_LOADING_TIME_MILLIS
import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport
import net.mullvad.mullvadvpn.dataproxy.SendProblemReportResult
import net.mullvad.mullvadvpn.dataproxy.UserReport

data class ReportProblemUiState(
    val showConfirmNoEmail: Boolean = false,
    val sendingState: SendingReportUiState? = null
)

sealed interface SendingReportUiState {
    data object Sending : SendingReportUiState

    data class Success(val email: String?) : SendingReportUiState

    data class Error(val error: SendProblemReportResult.Error) : SendingReportUiState
}

class ReportProblemViewModel(private val mullvadProblemReporter: MullvadProblemReport) :
    ViewModel() {

    private val _uiState = MutableStateFlow(ReportProblemUiState())
    val uiState = _uiState.asStateFlow()

    fun sendReport(
        email: String,
        description: String,
    ) {
        viewModelScope.launch {
            val userEmail = email.trim()
            val nullableEmail = if (email.isEmpty()) null else userEmail
            if (shouldShowConfirmNoEmail(nullableEmail)) {
                _uiState.update { it.copy(showConfirmNoEmail = true) }
            } else {
                _uiState.update {
                    it.copy(sendingState = SendingReportUiState.Sending, showConfirmNoEmail = false)
                }

                // Ensure we show loading for at least MINIMUM_LOADING_TIME_MILLIS
                val deferredResult = async {
                    mullvadProblemReporter.sendReport(UserReport(nullableEmail, description))
                }
                delay(MINIMUM_LOADING_TIME_MILLIS)

                _uiState.update {
                    it.copy(sendingState = deferredResult.await().toUiResult(nullableEmail))
                }
            }
        }
    }

    fun clearSendResult() {
        _uiState.update { it.copy(sendingState = null) }
    }

    fun dismissConfirmNoEmail() {
        _uiState.update { it.copy(showConfirmNoEmail = false) }
    }

    private fun shouldShowConfirmNoEmail(userEmail: String?): Boolean =
        userEmail.isNullOrEmpty() &&
            !uiState.value.showConfirmNoEmail &&
            uiState.value.sendingState !is SendingReportUiState

    private fun SendProblemReportResult.toUiResult(email: String?): SendingReportUiState =
        when (this) {
            is SendProblemReportResult.Error -> SendingReportUiState.Error(this)
            SendProblemReportResult.Success -> SendingReportUiState.Success(email)
        }

    init {
        viewModelScope.launch { mullvadProblemReporter.collectLogs() }
    }

    override fun onCleared() {
        super.onCleared()
        // Delete any logs if user leaves the screen
        mullvadProblemReporter.deleteLogs()
    }
}
