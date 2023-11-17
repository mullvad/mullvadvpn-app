package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.async
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.constant.MINIMUM_LOADING_TIME_MILLIS
import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport
import net.mullvad.mullvadvpn.dataproxy.SendProblemReportResult
import net.mullvad.mullvadvpn.dataproxy.UserReport
import net.mullvad.mullvadvpn.repository.ProblemReportRepository

data class ReportProblemUiState(
    val showConfirmNoEmail: Boolean = false,
    val sendingState: SendingReportUiState? = null,
    val email: String = "",
    val description: String = "",
)

sealed interface SendingReportUiState {
    data object Sending : SendingReportUiState

    data class Success(val email: String?) : SendingReportUiState

    data class Error(val error: SendProblemReportResult.Error) : SendingReportUiState
}

class ReportProblemViewModel(
    private val mullvadProblemReporter: MullvadProblemReport,
    private val problemReportRepository: ProblemReportRepository
) : ViewModel() {

    private val showConfirmNoEmail = MutableStateFlow(false)
    private val sendingState: MutableStateFlow<SendingReportUiState?> = MutableStateFlow(null)

    val uiState =
        combine(
                showConfirmNoEmail,
                sendingState,
                problemReportRepository.problemReport,
            ) { showConfirmNoEmail, pendingState, userReport ->
                ReportProblemUiState(
                    showConfirmNoEmail = showConfirmNoEmail,
                    sendingState = pendingState,
                    email = userReport.email ?: "",
                    description = userReport.description,
                )
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), ReportProblemUiState())

    fun sendReport(
        email: String,
        description: String,
    ) {
        viewModelScope.launch {
            val userEmail = email.trim()
            val nullableEmail = if (email.isEmpty()) null else userEmail
            if (shouldShowConfirmNoEmail(nullableEmail)) {
                showConfirmNoEmail.tryEmit(true)
            } else {
                sendingState.tryEmit(SendingReportUiState.Sending)
                showConfirmNoEmail.tryEmit(false)

                // Ensure we show loading for at least MINIMUM_LOADING_TIME_MILLIS
                val deferredResult = async {
                    mullvadProblemReporter.sendReport(UserReport(nullableEmail, description))
                }
                delay(MINIMUM_LOADING_TIME_MILLIS)

                val result = deferredResult.await()
                // Clear saved problem report if report was sent successfully
                if (result is SendProblemReportResult.Success) {
                    problemReportRepository.setEmail("")
                    problemReportRepository.setDescription("")
                }
                sendingState.tryEmit(deferredResult.await().toUiResult(nullableEmail))
            }
        }
    }

    fun clearSendResult() {
        sendingState.tryEmit(null)
    }

    fun dismissConfirmNoEmail() {
        showConfirmNoEmail.tryEmit(false)
    }

    fun updateEmail(email: String) {
        problemReportRepository.setEmail(email)
    }

    fun updateDescription(description: String) {
        problemReportRepository.setDescription(description)
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
