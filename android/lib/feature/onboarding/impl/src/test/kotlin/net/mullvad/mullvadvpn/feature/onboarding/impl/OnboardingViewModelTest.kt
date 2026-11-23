package net.mullvad.mullvadvpn.feature.onboarding.impl

import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.userpreferences.OnboardingCheckpoint
import net.mullvad.mullvadvpn.lib.userpreferences.UserPreferencesRepository
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class OnboardingViewModelTest {

    private val mockPreferencesRepository: UserPreferencesRepository = mockk(relaxed = true)
    private val viewModel = TestOnboardingViewModel(mockPreferencesRepository)

    private class TestOnboardingViewModel(
        repository: UserPreferencesRepository,
    ) : OnboardingViewModel(repository) {
        fun enterWithCheckpoint() = persistCheckpoint(OnboardingCheckpoint.GOOD_TO_KNOW)

        fun markPermissionAsked() = markNotificationPermissionAsked()
    }

    @Test
    fun `when flow completes should mark the onboarding as completed`() = runTest {
        // Act
        viewModel.complete()

        // Assert
        coVerify {
            mockPreferencesRepository.setOnboardingCheckpoint(OnboardingCheckpoint.PART_1_COMPLETED)
        }
    }

    @Test
    fun `when a step is entered should persist its checkpoint`() = runTest {
        // Act
        viewModel.enterWithCheckpoint()

        // Assert
        coVerify {
            mockPreferencesRepository.setOnboardingCheckpoint(OnboardingCheckpoint.GOOD_TO_KNOW)
        }
    }

    @Test
    fun `when notifications are dismissed should mark permission as asked`() = runTest {
        // Act
        viewModel.markPermissionAsked()

        // Assert
        coVerify { mockPreferencesRepository.setNotificationPermissionAsked() }
    }
}
