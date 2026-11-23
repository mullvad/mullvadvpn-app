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
class NotificationsViewModelTest {

    private val mockPreferencesRepository: UserPreferencesRepository = mockk(relaxed = true)
    private val viewModel = NotificationsViewModel(mockPreferencesRepository)

    @Test
    fun `when the step is entered should persist the notifications checkpoint`() = runTest {

        // Assert
        coVerify {
            mockPreferencesRepository.setOnboardingCheckpoint(OnboardingCheckpoint.NOTIFICATIONS)
        }
    }

    @Test
    fun `when the step is dismissed should mark permission as asked`() = runTest {
        // Act
        viewModel.onNotificationStepDismissed()

        // Assert
        coVerify { mockPreferencesRepository.setNotificationPermissionAsked() }
    }
}
