package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.repository.ProblemReportRepository
import net.mullvad.mullvadvpn.lib.ui.component.NEWLINE_STRING

data class ViewLogsUiState(val allLines: List<String> = emptyList()) {
    fun text() = allLines.joinToString(NEWLINE_STRING)
}

class ViewLogsViewModel(private val problemReportRepository: ProblemReportRepository) : ViewModel() {

    private val _uiState = MutableStateFlow<Lc<Unit, ViewLogsUiState>>(Lc.Loading(Unit))
    val uiState = _uiState.asStateFlow()

    init {
        viewModelScope.launch {
            _uiState.update {
                Lc.Content(ViewLogsUiState(allLines = problemReportRepository.readLogs()))
            }
        }
    }
}
