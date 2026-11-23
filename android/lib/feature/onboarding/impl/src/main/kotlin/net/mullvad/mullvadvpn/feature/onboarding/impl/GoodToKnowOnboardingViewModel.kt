package net.mullvad.mullvadvpn.feature.onboarding.impl

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.feature.onboarding.api.GoodToKnowStep
import net.mullvad.mullvadvpn.lib.userpreferences.OnboardingCheckpoint
import net.mullvad.mullvadvpn.lib.userpreferences.UserPreferencesRepository

data class GoodToKnowUiState(val page: GoodToKnowOnboardingPage)

class GoodToKnowOnboardingViewModel(
    private val userPreferencesRepository: UserPreferencesRepository,
    step: GoodToKnowStep,
) : ViewModel() {

    init {
        if (step == GoodToKnowStep.GoodToKnow1) {
            viewModelScope.launch {
                userPreferencesRepository.setOnboardingCheckpoint(OnboardingCheckpoint.GOOD_TO_KNOW)
            }
        }
    }

    val uiState: StateFlow<GoodToKnowUiState> =
        MutableStateFlow(GoodToKnowUiState(page = GoodToKnowOnboardingPage.forStep(step)))

    fun complete() = viewModelScope.launch {
        userPreferencesRepository.setOnboardingCheckpoint(OnboardingCheckpoint.PART_1_COMPLETED)
    }
}
