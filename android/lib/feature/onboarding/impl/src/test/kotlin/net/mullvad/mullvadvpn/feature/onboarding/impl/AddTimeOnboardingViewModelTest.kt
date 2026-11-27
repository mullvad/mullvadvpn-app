package net.mullvad.mullvadvpn.feature.onboarding.impl

import androidx.lifecycle.Lifecycle
import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import java.time.ZonedDateTime
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.runAndCancelContextTest
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.PaymentLogic
import net.mullvad.mullvadvpn.lib.userpreferences.OnboardingCheckpoint
import net.mullvad.mullvadvpn.lib.userpreferences.UserPreferencesRepository
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class AddTimeOnboardingViewModelTest {

    private val accountData = MutableStateFlow<AccountData?>(null)
    private val onboardingCheckpoint = MutableStateFlow(OnboardingCheckpoint.NOT_STARTED)
    private val mockAccountRepository: AccountRepository = mockk(relaxed = true)
    private val mockPreferencesRepository: UserPreferencesRepository = mockk(relaxed = true)
    private val mockPaymentUseCase: PaymentLogic = mockk(relaxed = true)
    private val mockActivityLifecycle: Lifecycle = mockk(relaxed = true)
    private val paymentAvailabilityFlow = MutableStateFlow<PaymentAvailability?>(null)

    private lateinit var viewModel: AddTimeOnboardingViewModel

    @BeforeEach
    fun setup() {
        every { mockAccountRepository.accountData } returns accountData
        every { mockPaymentUseCase.paymentAvailability } returns flowOf(null)
        every { mockPreferencesRepository.onboardingCheckpoint() } returns onboardingCheckpoint
        coEvery { mockPaymentUseCase.paymentAvailability } returns paymentAvailabilityFlow

        viewModel =
            AddTimeOnboardingViewModel(
                accountRepository = mockAccountRepository,
                paymentUseCase = mockPaymentUseCase,
                userPreferencesRepository = mockPreferencesRepository,
                activityLifecycle = mockActivityLifecycle,
            )
    }

    private fun creditedAccount(): AccountData =
        mockk<AccountData>().apply { every { expiryDate } returns ZonedDateTime.now().plusDays(30) }

    @Test
    fun `when time is added should emit TimeAdded and reset purchase state`() =
        runAndCancelContextTest(viewModel.viewModelScope.coroutineContext) {
            // Act, Assert
            viewModel.uiSideEffect.test {
                accountData.value = creditedAccount()
                val effect =
                    assertIs<AddTimeOnboardingViewModel.UiSideEffect.TimeAdded>(awaitItem())
                assertEquals(false, effect.hasCompletedOnboarding)
                cancelAndIgnoreRemainingEvents()
            }
            coVerify { mockPaymentUseCase.resetPurchaseResult() }
            verify { mockAccountRepository.resetIsNewAccount() }
            // Adding time does not complete the onboarding flow.
            coVerify(exactly = 0) {
                mockPreferencesRepository.setOnboardingCheckpoint(OnboardingCheckpoint.COMPLETED)
            }
        }

    @Test
    fun `when the account already has time should emit TimeAdded`() =
        runAndCancelContextTest(viewModel.viewModelScope.coroutineContext) {
            // Arrange
            accountData.value = creditedAccount()

            // Act, Assert
            viewModel.uiSideEffect.test {
                assertIs<AddTimeOnboardingViewModel.UiSideEffect.TimeAdded>(awaitItem())
                cancelAndIgnoreRemainingEvents()
            }
        }

    @Test
    fun `when onboarding is already completed should flag it on TimeAdded`() =
        runAndCancelContextTest(viewModel.viewModelScope.coroutineContext) {
            // Arrange
            onboardingCheckpoint.value = OnboardingCheckpoint.COMPLETED
            accountData.value = creditedAccount()

            // Act, Assert
            viewModel.uiSideEffect.test {
                val effect =
                    assertIs<AddTimeOnboardingViewModel.UiSideEffect.TimeAdded>(awaitItem())
                assertEquals(true, effect.hasCompletedOnboarding)
                cancelAndIgnoreRemainingEvents()
            }
        }

    @Test
    fun `when account is still expired should not emit TimeAdded`() =
        runAndCancelContextTest(viewModel.viewModelScope.coroutineContext) {
            // Arrange
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
    fun `when there is a pending purchase, uiState should reflect it`() =
        runAndCancelContextTest(viewModel.viewModelScope.coroutineContext) {
            // Arrange
            paymentAvailabilityFlow.value =
                PaymentAvailability.ProductsAvailable(
                    products =
                        listOf(
                            PaymentProduct(
                                productId = ProductId("test_product_id"),
                                price = ProductPrice("9.99"),
                                status = PaymentStatus.PENDING,
                            ),
                        ),
                )

            // Act, Assert
            viewModel.uiState.test {
                val result = awaitItem()
                assertIs<AddTimeOnboardingUiState>(result)
                assertEquals(PaymentStatus.PENDING, result.paymentStatus)
            }
        }
}
