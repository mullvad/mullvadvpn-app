package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import net.mullvad.mullvadvpn.dataproxy.UserReport

class ProblemReportRepository {
    private val _problemReport = MutableStateFlow(UserReport("", ""))
    val problemReport: StateFlow<UserReport> = _problemReport.asStateFlow()

    fun setEmail(email: String) = _problemReport.update { it.copy(email = email) }

    fun setDescription(description: String) =
        _problemReport.update { it.copy(description = description) }
}
