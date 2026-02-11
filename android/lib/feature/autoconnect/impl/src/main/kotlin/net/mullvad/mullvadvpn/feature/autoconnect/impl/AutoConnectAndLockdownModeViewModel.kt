package net.mullvad.mullvadvpn.feature.autoconnect.impl

import androidx.lifecycle.ViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

class AutoConnectAndLockdownModeViewModel(val isPlayBuild: Boolean) : ViewModel() {
    val uiState: StateFlow<AutoConnectAndLockdownModeUiState> =
        MutableStateFlow(AutoConnectAndLockdownModeUiState(isPlayBuild))
}
