package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport

data class ViewLogsUiState(val allLines: List<String> = emptyList(), val isLoading: Boolean = true)

class ViewLogsViewModel(private val mullvadProblemReporter: MullvadProblemReport) : ViewModel() {

    private val _uiState = MutableStateFlow(ViewLogsUiState())
    val uiState = _uiState.asStateFlow()

    init {
        viewModelScope.launch {
            _uiState.update {
                it.copy(allLines = mullvadProblemReporter.readLogs(), isLoading = false)
            }
        }
    }
}
