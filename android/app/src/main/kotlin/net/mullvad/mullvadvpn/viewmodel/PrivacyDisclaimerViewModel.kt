package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.repository.PrivacyDisclaimerRepository

class PrivacyDisclaimerViewModel(
    private val privacyDisclaimerRepository: PrivacyDisclaimerRepository
) : ViewModel() {

    private val _uiSideEffect =
        Channel<PrivacyDisclaimerUiSideEffect>(1, BufferOverflow.DROP_OLDEST)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun setPrivacyDisclosureAccepted() {
        privacyDisclaimerRepository.setPrivacyDisclosureAccepted()
        viewModelScope.launch { _uiSideEffect.send(PrivacyDisclaimerUiSideEffect.NavigateToLogin) }
    }
}

sealed interface PrivacyDisclaimerUiSideEffect {
    data object NavigateToLogin : PrivacyDisclaimerUiSideEffect
}
