package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import net.mullvad.mullvadvpn.dataproxy.UserReport

class ProblemReportRepository {
    private val _problemReport = MutableStateFlow(UserReport("", ""))
    val problemReport: StateFlow<UserReport> = _problemReport.asStateFlow()

    fun setEmail(email: String) {
        _problemReport.value = _problemReport.value.copy(email = email)
    }

    fun setDescription(description: String) {
        _problemReport.value = _problemReport.value.copy(description = description)
    }
}
