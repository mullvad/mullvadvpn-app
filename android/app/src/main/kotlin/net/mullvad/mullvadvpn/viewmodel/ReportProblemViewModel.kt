package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.async
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.common.compose.MINIMUM_LOADING_TIME_MILLIS
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.UserReport
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.ProblemReportRepository
import net.mullvad.mullvadvpn.lib.repository.SendProblemReportResult
import net.mullvad.mullvadvpn.util.combine

data class ReportProblemUiState(
    val sendingState: SendingReportUiState? = null,
    val email: String = "",
    val description: String = "",
    val showIncludeAccountId: Boolean = false,
    val includeAccountId: Boolean = false,
    val showIncludeAccountWarningMessage: Boolean = false,
    val logCollectingState: LogCollectingState = LogCollectingState.Loading,
    val isPlayBuild: Boolean = false,
)

sealed interface SendingReportUiState {
    data object Sending : SendingReportUiState

    data class Success(val email: String?) : SendingReportUiState

    data class Error(val error: SendProblemReportResult.Error) : SendingReportUiState
}

sealed interface LogCollectingState {
    data object Loading : LogCollectingState

    data object Success : LogCollectingState

    data object Failed : LogCollectingState
}

sealed interface ReportProblemSideEffect {
    data object ShowConfirmNoEmail : ReportProblemSideEffect
}

class ReportProblemViewModel(
    private val mullvadProblemReporter: ProblemReportRepository,
    private val problemReportRepository: ProblemReportRepository,
    accountRepository: AccountRepository,
    private val isPlayBuild: Boolean,
) : ViewModel() {

    private val sendingState: MutableStateFlow<SendingReportUiState?> = MutableStateFlow(null)
    private val includeAccountIdState: MutableStateFlow<Boolean> = MutableStateFlow(false)
    private val showIncludeAccountWarningMessage: MutableStateFlow<Boolean> =
        MutableStateFlow(false)
    private val areLogsCollected: MutableStateFlow<LogCollectingState> =
        MutableStateFlow(LogCollectingState.Loading)

    val uiState =
        combine(
                sendingState,
                includeAccountIdState,
                showIncludeAccountWarningMessage,
                problemReportRepository.problemReport,
                accountRepository.accountData,
                areLogsCollected,
            ) {
                sendingState,
                includeAccountToken,
                showIncludeAccountWarningMessage,
                userReport,
                accountData,
                areLogsCollected ->
                ReportProblemUiState(
                    sendingState = sendingState,
                    email = userReport.email ?: "",
                    description = userReport.description,
                    showIncludeAccountId = accountData != null,
                    includeAccountId = includeAccountToken,
                    showIncludeAccountWarningMessage = showIncludeAccountWarningMessage,
                    logCollectingState = areLogsCollected,
                    isPlayBuild = isPlayBuild,
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                ReportProblemUiState(isPlayBuild = isPlayBuild),
            )

    private val _uiSideEffect = Channel<ReportProblemSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun sendReport(email: String, description: String, skipEmptyEmailCheck: Boolean = false) {
        viewModelScope.launch {
            val userEmail = email.trim()
            val nullableEmail = if (email.isEmpty()) null else userEmail
            if (!skipEmptyEmailCheck && shouldShowConfirmNoEmail(nullableEmail)) {
                _uiSideEffect.send(ReportProblemSideEffect.ShowConfirmNoEmail)
            } else {
                sendingState.emit(SendingReportUiState.Sending)

                // Ensure we show loading for at least MINIMUM_LOADING_TIME_MILLIS
                val deferredResult = async {
                    mullvadProblemReporter.sendReport(
                        UserReport(nullableEmail, description),
                        includeAccountIdState.value,
                    )
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

    fun updateEmail(email: String) {
        problemReportRepository.setEmail(email)
    }

    fun updateDescription(description: String) {
        problemReportRepository.setDescription(description)
    }

    fun onIncludeAccountIdCheckChange(checked: Boolean) {
        includeAccountIdState.tryEmit(checked)
    }

    fun showIncludeAccountInformationWarningMessage(show: Boolean) {
        showIncludeAccountWarningMessage.tryEmit(show)
    }

    private fun shouldShowConfirmNoEmail(userEmail: String?): Boolean =
        userEmail.isNullOrEmpty() && uiState.value.sendingState !is SendingReportUiState

    private fun SendProblemReportResult.toUiResult(email: String?): SendingReportUiState =
        when (this) {
            is SendProblemReportResult.Error -> SendingReportUiState.Error(this)
            SendProblemReportResult.Success -> SendingReportUiState.Success(email)
        }

    init {
        viewModelScope.launch {
            if (mullvadProblemReporter.collectLogs()) {
                areLogsCollected.emit(LogCollectingState.Success)
            } else {
                areLogsCollected.emit(LogCollectingState.Failed)
            }
        }
    }

    override fun onCleared() {
        super.onCleared()
        // Delete any logs if user leaves the screen
        mullvadProblemReporter.deleteLogs()
    }
}
