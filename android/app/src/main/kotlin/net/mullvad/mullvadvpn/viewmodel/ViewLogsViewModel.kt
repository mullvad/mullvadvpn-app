package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.constant.NAVIGATION_DELAY_MILLIS
import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport

data class ViewLogsUiState(val allLines: String = "", val isLoading: Boolean = true)

class ViewLogsViewModel(private val mullvadProblemReporter: MullvadProblemReport) : ViewModel() {

    private val _uiState = MutableStateFlow(ViewLogsUiState())
    val uiState = _uiState.asStateFlow()

    init {
        viewModelScope.launch {
            // Loading this much text takes a while, so we show a loading indicator until the
            // fragment transitions is done. I'd very much prefer to use LazyColumn in the view
            // which would make the loading way faster but then the SelectionContainer is broken and
            // text would not be copyable.
            delay(NAVIGATION_DELAY_MILLIS)
            _uiState.update {
                it.copy(
                    allLines = mullvadProblemReporter.readLogs().joinToString("\n"),
                    isLoading = false
                )
            }
        }
    }
}
