package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.shared.UserPreferencesRepository

data class PrivacyDisclaimerViewState(val isStartingService: Boolean, val isPlayBuild: Boolean)

class PrivacyDisclaimerViewModel(
    private val userPreferencesRepository: UserPreferencesRepository,
    isPlayBuild: Boolean,
) : ViewModel() {

    private val _isStartingService = MutableStateFlow(false)

    val uiState =
        _isStartingService
            .map { isStartingService ->
                PrivacyDisclaimerViewState(
                    isStartingService = isStartingService,
                    isPlayBuild = isPlayBuild,
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                PrivacyDisclaimerViewState(false, isPlayBuild),
            )

    private val _uiSideEffect = Channel<PrivacyDisclaimerUiSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun setPrivacyDisclosureAccepted() {
        viewModelScope.launch {
            userPreferencesRepository.setPrivacyDisclosureAccepted()
            if (!_isStartingService.value) {
                _isStartingService.update { true }
                _uiSideEffect.send(PrivacyDisclaimerUiSideEffect.StartService)
            }
        }
    }

    fun onServiceStartedSuccessful() {
        viewModelScope.launch {
            _uiSideEffect.send(PrivacyDisclaimerUiSideEffect.NavigateToLogin)
            _isStartingService.update { false }
        }
    }

    fun onServiceStartedTimeout() {
        viewModelScope.launch {
            _uiSideEffect.send(PrivacyDisclaimerUiSideEffect.NavigateToSplash)
            _isStartingService.update { false }
        }
    }
}

sealed interface PrivacyDisclaimerUiSideEffect {
    data object NavigateToLogin : PrivacyDisclaimerUiSideEffect

    data object StartService : PrivacyDisclaimerUiSideEffect

    data object NavigateToSplash : PrivacyDisclaimerUiSideEffect
}
