package net.mullvad.mullvadvpn.feature.onboarding.impl

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.userpreferences.OnboardingCheckpoint
import net.mullvad.mullvadvpn.lib.userpreferences.UserPreferencesRepository

abstract class OnboardingViewModel(
    private val userPreferencesRepository: UserPreferencesRepository,
) : ViewModel() {

    protected fun persistCheckpoint(checkpoint: OnboardingCheckpoint) = viewModelScope.launch {
        userPreferencesRepository.setOnboardingCheckpoint(checkpoint)
    }

    protected fun markNotificationPermissionAsked() = viewModelScope.launch {
        userPreferencesRepository.setNotificationPermissionAsked()
    }

    fun complete() = persistCheckpoint(OnboardingCheckpoint.PART_1_COMPLETED)
}
