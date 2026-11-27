package net.mullvad.mullvadvpn.feature.onboarding.impl

import io.mockk.coVerify
import io.mockk.mockk
import kotlin.test.assertEquals
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.feature.onboarding.api.GoodToKnowStep
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.userpreferences.OnboardingCheckpoint
import net.mullvad.mullvadvpn.lib.userpreferences.UserPreferencesRepository
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class GoodToKnowOnboardingViewModelTest {

    private val mockPreferencesRepository: UserPreferencesRepository = mockk(relaxed = true)

    @Test
    fun `first page should expose blocked content and persist the good to know checkpoint`() =
        runTest {
            // Arrange
            val viewModel =
                GoodToKnowOnboardingViewModel(mockPreferencesRepository, GoodToKnowStep.GoodToKnow1)

            // Act, Assert
            assertEquals(GoodToKnowOnboardingPage.BLOCKED_CONTENT, viewModel.uiState.value.page)
            coVerify {
                mockPreferencesRepository.setOnboardingCheckpoint(
                    OnboardingCheckpoint.GOOD_TO_KNOW,
                )
            }
        }

    @Test
    fun `second page should expose excluding apps and not change the checkpoint`() = runTest {
        // Arrange
        val viewModel =
            GoodToKnowOnboardingViewModel(mockPreferencesRepository, GoodToKnowStep.GoodToKnow2)

        // Act, Assert
        assertEquals(GoodToKnowOnboardingPage.EXCLUDING_APPS, viewModel.uiState.value.page)
        coVerify(exactly = 0) { mockPreferencesRepository.setOnboardingCheckpoint(any()) }
    }
}
