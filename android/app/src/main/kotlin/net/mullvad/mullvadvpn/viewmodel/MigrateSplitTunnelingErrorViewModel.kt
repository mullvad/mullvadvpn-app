package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.repository.SplitTunnelingRepository

class MigrateSplitTunnelingErrorViewModel(
    private val splitTunnelingRepository: SplitTunnelingRepository
) : ViewModel() {
    private val _uiSideEffect = Channel<MigrateSplitTunnelingErrorUiSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun clearOldSettings() {
        viewModelScope.launch {
            splitTunnelingRepository.clearOldSettings()
            _uiSideEffect.send(MigrateSplitTunnelingErrorUiSideEffect.CloseScreen)
        }
    }

    fun tryAgainLater() {
        viewModelScope.launch {
            splitTunnelingRepository.resetShouldTryMigrateSplitTunneling()
            _uiSideEffect.send(MigrateSplitTunnelingErrorUiSideEffect.CloseScreen)
        }
    }
}

interface MigrateSplitTunnelingErrorUiSideEffect {
    data object CloseScreen : MigrateSplitTunnelingErrorUiSideEffect
}
