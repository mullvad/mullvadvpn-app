package net.mullvad.mullvadvpn.feature.onboarding.impl

import app.cash.turbine.test
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import java.time.ZonedDateTime
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.feature.onboarding.api.GoodToKnowStep
import net.mullvad.mullvadvpn.feature.onboarding.api.OnboardingStep
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.PaymentLogic
import net.mullvad.mullvadvpn.lib.usecase.SelectedLocationTitleUseCase
import net.mullvad.mullvadvpn.lib.userpreferences.OnboardingCheckpoint
import net.mullvad.mullvadvpn.lib.userpreferences.UserPreferencesRepository
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class OnboardingViewModelTest {

    private val accountData = MutableStateFlow<AccountData?>(null)
    private val onboardingCheckpoint = MutableStateFlow(OnboardingCheckpoint.NOT_STARTED)
    private val mockAccountRepository: AccountRepository = mockk(relaxed = true)
    private val mockPreferencesRepository: UserPreferencesRepository = mockk(relaxed = true)
    private val mockPaymentUseCase: PaymentLogic = mockk(relaxed = true)
    private val mockSelectedLocationTitleUseCase: SelectedLocationTitleUseCase = mockk()

    private lateinit var viewModel: OnboardingViewModel

    @BeforeEach
    fun setup() {
        every { mockAccountRepository.accountData } returns accountData
        every { mockSelectedLocationTitleUseCase() } returns flowOf(null)
        every { mockPreferencesRepository.onboardingCheckpoint() } returns onboardingCheckpoint
        viewModel = createViewModel()
    }

    private fun createViewModel() =
        OnboardingViewModel(
            accountRepository = mockAccountRepository,
            userPreferencesRepository = mockPreferencesRepository,
            paymentUseCase = mockPaymentUseCase,
            selectedLocationTitleUseCase = mockSelectedLocationTitleUseCase,
        )

    private fun creditedAccount(): AccountData =
        mockk<AccountData>().apply { every { expiryDate } returns ZonedDateTime.now().plusDays(30) }

    @Test
    fun `when time is added on the add time step should emit TimeAdded`() = runTest {
        // Arrange
        viewModel.onStepEntered(OnboardingStep.AddTime)

        // Act, Assert
        viewModel.uiSideEffect.test {
            accountData.value = creditedAccount()
            val effect = assertIs<OnboardingViewModel.UiSideEffect.TimeAdded>(awaitItem())
            assertEquals(false, effect.hasCompletedOnboarding)
            cancelAndIgnoreRemainingEvents()
        }
        // The onboarding flow continues, it is not completed by added time.
        coVerify(exactly = 0) {
            mockPreferencesRepository.setOnboardingCheckpoint(OnboardingCheckpoint.COMPLETED)
        }
        coVerify { mockPaymentUseCase.resetPurchaseResult() }
        verify { mockAccountRepository.resetIsNewAccount() }
    }

    @Test
    fun `when the account already has time as the add time step is entered should emit TimeAdded`() =
        runTest {
            // Arrange
            accountData.value = creditedAccount()

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.onStepEntered(OnboardingStep.AddTime)
                assertIs<OnboardingViewModel.UiSideEffect.TimeAdded>(awaitItem())
                cancelAndIgnoreRemainingEvents()
            }
        }

    @Test
    fun `when onboarding is already completed should flag it on TimeAdded`() = runTest {
        // Arrange
        onboardingCheckpoint.value = OnboardingCheckpoint.COMPLETED
        accountData.value = creditedAccount()

        // Act, Assert
        viewModel.uiSideEffect.test {
            viewModel.onStepEntered(OnboardingStep.AddTime)
            val effect = assertIs<OnboardingViewModel.UiSideEffect.TimeAdded>(awaitItem())
            assertEquals(true, effect.hasCompletedOnboarding)
            cancelAndIgnoreRemainingEvents()
        }
    }

    @Test
    fun `when account is still expired should not emit TimeAdded`() = runTest {
        // Arrange
        viewModel.onStepEntered(OnboardingStep.AddTime)
        val expiredAccount: AccountData = mockk()
        every { expiredAccount.expiryDate } returns ZonedDateTime.now().minusDays(1)

        // Act, Assert
        viewModel.uiSideEffect.test {
            accountData.value = expiredAccount
            expectNoEvents()
            cancelAndIgnoreRemainingEvents()
        }
    }

    @Test
    fun `when time is added on a later step should not emit TimeAdded`() = runTest {
        // Arrange
        viewModel.onStepEntered(OnboardingStep.Notifications)

        // Act, Assert
        viewModel.uiSideEffect.test {
            accountData.value = creditedAccount()
            expectNoEvents()
            cancelAndIgnoreRemainingEvents()
        }
    }

    @Test
    fun `when notifications step is entered should persist the notifications checkpoint`() =
        runTest {
            // Act
            viewModel.onStepEntered(OnboardingStep.Notifications)

            // Assert
            coVerify {
                mockPreferencesRepository.setOnboardingCheckpoint(
                    OnboardingCheckpoint.NOTIFICATIONS,
                )
            }
        }

    @Test
    fun `when first good to know step is entered should persist the good to know checkpoint`() =
        runTest {
            // Act
            viewModel.onStepEntered(GoodToKnowStep.GoodToKnow1)

            // Assert
            coVerify {
                mockPreferencesRepository.setOnboardingCheckpoint(
                    OnboardingCheckpoint.GOOD_TO_KNOW,
                )
            }
        }

    @Test
    fun `when add time step is entered should not change the checkpoint`() = runTest {
        // Act
        viewModel.onStepEntered(OnboardingStep.AddTime)

        // Assert
        coVerify(exactly = 0) { mockPreferencesRepository.setOnboardingCheckpoint(any()) }
    }

    @Test
    fun `when notifications step is dismissed should mark permission as asked`() = runTest {
        // Act
        viewModel.onNotificationStepDismissed()

        // Assert
        coVerify { mockPreferencesRepository.setNotificationPermissionAsked() }
    }

    @Test
    fun `when flow completes should mark the onboarding as completed`() = runTest {
        // Act
        viewModel.complete()

        // Assert
        coVerify {
            mockPreferencesRepository.setOnboardingCheckpoint(OnboardingCheckpoint.COMPLETED)
        }
    }

    @Test
    fun `uiState should expose the selected location`() = runTest {
        // Arrange
        val selectedLocation = MutableStateFlow<String?>(null)
        every { mockSelectedLocationTitleUseCase() } returns selectedLocation
        val newViewModel = createViewModel()

        // Act, Assert
        newViewModel.uiState.test {
            assertEquals(null, awaitItem().selectedLocationTitle)
            selectedLocation.value = "Sweden"
            assertEquals("Sweden", awaitItem().selectedLocationTitle)
            cancelAndIgnoreRemainingEvents()
        }
    }
}
