package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.repository.MigrateSplitTunnelingRepository

class MigrateSplitTunnelingErrorViewModel(
    private val migrateSplitTunnelingRepository: MigrateSplitTunnelingRepository
) : ViewModel() {
    private val _uiSideEffect = Channel<MigrateSplitTunnelingErrorUiSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun clearOldSettings() {
        viewModelScope.launch {
            migrateSplitTunnelingRepository.clearOldSettings()
            _uiSideEffect.send(MigrateSplitTunnelingErrorUiSideEffect.CloseScreen)
        }
    }

    fun tryAgain() {
        viewModelScope.launch {
            migrateSplitTunnelingRepository.migrateSplitTunneling()
            _uiSideEffect.send(MigrateSplitTunnelingErrorUiSideEffect.CloseScreen)
        }
    }
}

interface MigrateSplitTunnelingErrorUiSideEffect {
    data object CloseScreen : MigrateSplitTunnelingErrorUiSideEffect
}
