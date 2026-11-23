package net.mullvad.mullvadvpn.feature.onboarding.impl

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import net.mullvad.mullvadvpn.feature.onboarding.api.GoodToKnowStep
import net.mullvad.mullvadvpn.lib.userpreferences.OnboardingCheckpoint
import net.mullvad.mullvadvpn.lib.userpreferences.UserPreferencesRepository

data class GoodToKnowUiState(val page: GoodToKnowPage)

class GoodToKnowViewModel(
    userPreferencesRepository: UserPreferencesRepository,
    step: GoodToKnowStep,
) : OnboardingViewModel(userPreferencesRepository) {

    init {
        if (step == GoodToKnowStep.GoodToKnow1) {
            persistCheckpoint(OnboardingCheckpoint.GOOD_TO_KNOW)
        }
    }

    val uiState: StateFlow<GoodToKnowUiState> =
        MutableStateFlow(GoodToKnowUiState(page = GoodToKnowPage.forStep(step)))
}
