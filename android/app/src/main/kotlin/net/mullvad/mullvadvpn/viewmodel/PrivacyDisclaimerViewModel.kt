package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.repository.PrivacyDisclaimerRepository

class PrivacyDisclaimerViewModel(
    private val privacyDisclaimerRepository: PrivacyDisclaimerRepository
) : ViewModel() {
    private val _uiSideEffect =
        MutableSharedFlow<PrivacyDisclaimerUiSideEffect>(extraBufferCapacity = 1)
    val uiSideEffect = _uiSideEffect.asSharedFlow()

    fun setPrivacyDisclosureAccepted() {
        privacyDisclaimerRepository.setPrivacyDisclosureAccepted()
        viewModelScope.launch { _uiSideEffect.emit(PrivacyDisclaimerUiSideEffect.NavigateToLogin) }
    }
}

sealed interface PrivacyDisclaimerUiSideEffect {
    data object NavigateToLogin : PrivacyDisclaimerUiSideEffect
}
