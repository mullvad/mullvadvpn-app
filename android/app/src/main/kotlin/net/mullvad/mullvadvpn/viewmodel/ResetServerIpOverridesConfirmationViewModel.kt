package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.repository.RelayOverridesRepository

class ResetServerIpOverridesConfirmationViewModel(
    private val relayOverridesRepository: RelayOverridesRepository,
) : ViewModel() {
    private val _uiSideEffect = Channel<ResetServerIpOverridesConfirmationUiSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun clearAllOverrides() =
        viewModelScope.launch {
            relayOverridesRepository.clearAllOverrides()
            _uiSideEffect.send(ResetServerIpOverridesConfirmationUiSideEffect.OverridesCleared)
        }
}

sealed class ResetServerIpOverridesConfirmationUiSideEffect {
    data object OverridesCleared : ResetServerIpOverridesConfirmationUiSideEffect()
}
