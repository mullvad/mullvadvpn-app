package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport
import net.mullvad.mullvadvpn.lib.ui.component.NEWLINE_STRING
import net.mullvad.mullvadvpn.repository.ApiAccessRepository

class ApiUnreachableViewModel(
    private val apiAccessRepository: ApiAccessRepository,
    private val mullvadProblemReport: MullvadProblemReport,
) : ViewModel() {

    private val _uiSideEffect = Channel<ApiUnreachableSideEffect>(Channel.BUFFERED)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun enableAllApiAccess() {
        viewModelScope.launch { apiAccessRepository.enableAllApiAccessMethods() }
    }

    fun sendProblemReportEmail() {
        viewModelScope.launch {
            val logs = mullvadProblemReport.readLogs()
            _uiSideEffect.send(
                ApiUnreachableSideEffect.SendEmail(logs = logs.joinToString(NEWLINE_STRING))
            )
        }
    }
}

sealed interface ApiUnreachableSideEffect {
    data class SendEmail(val logs: String) : ApiUnreachableSideEffect
}
