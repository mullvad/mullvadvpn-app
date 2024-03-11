package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.repository.PrivacyDisclaimerRepository

data class PrivacyDisclaimerViewState(val isStartingService: Boolean)

class PrivacyDisclaimerViewModel(
    private val privacyDisclaimerRepository: PrivacyDisclaimerRepository
) : ViewModel() {

    private val _uiState = MutableStateFlow(PrivacyDisclaimerViewState(false))
    val uiState = _uiState

    private val _uiSideEffect = Channel<PrivacyDisclaimerUiSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun setPrivacyDisclosureAccepted() {
        privacyDisclaimerRepository.setPrivacyDisclosureAccepted()
        viewModelScope.launch {
            _uiState.update { it.copy(isStartingService = true) }
            _uiSideEffect.send(PrivacyDisclaimerUiSideEffect.StartService)
        }
    }

    fun onServiceStartedSuccessful() {
        viewModelScope.launch { _uiSideEffect.send(PrivacyDisclaimerUiSideEffect.NavigateToLogin) }
    }

    fun onServiceStartedTimeout() {
        viewModelScope.launch { _uiSideEffect.send(PrivacyDisclaimerUiSideEffect.NavigateToSplash) }
    }
}

sealed interface PrivacyDisclaimerUiSideEffect {
    data object NavigateToLogin : PrivacyDisclaimerUiSideEffect

    data object StartService : PrivacyDisclaimerUiSideEffect

    data object NavigateToSplash : PrivacyDisclaimerUiSideEffect
}
