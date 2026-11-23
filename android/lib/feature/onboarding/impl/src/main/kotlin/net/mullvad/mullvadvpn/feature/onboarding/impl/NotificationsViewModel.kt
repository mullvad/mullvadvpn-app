package net.mullvad.mullvadvpn.feature.onboarding.impl

import net.mullvad.mullvadvpn.lib.userpreferences.OnboardingCheckpoint
import net.mullvad.mullvadvpn.lib.userpreferences.UserPreferencesRepository

class NotificationsViewModel(
    userPreferencesRepository: UserPreferencesRepository,
) : OnboardingViewModel(userPreferencesRepository) {

    init {
        persistCheckpoint(OnboardingCheckpoint.NOTIFICATIONS)
    }

    fun onNotificationStepDismissed() = markNotificationPermissionAsked()
}
