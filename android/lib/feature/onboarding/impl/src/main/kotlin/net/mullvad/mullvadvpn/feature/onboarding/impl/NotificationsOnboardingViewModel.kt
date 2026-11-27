package net.mullvad.mullvadvpn.feature.onboarding.impl

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.userpreferences.OnboardingCheckpoint
import net.mullvad.mullvadvpn.lib.userpreferences.UserPreferencesRepository

class NotificationsOnboardingViewModel(
    private val userPreferencesRepository: UserPreferencesRepository,
) : ViewModel() {

    init {
        viewModelScope.launch {
            userPreferencesRepository.setOnboardingCheckpoint(OnboardingCheckpoint.NOTIFICATIONS)
        }
    }

    fun onNotificationStepDismissed() = viewModelScope.launch {
        userPreferencesRepository.setNotificationPermissionAsked()
    }
}
