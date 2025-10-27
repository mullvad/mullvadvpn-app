package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import arrow.core.right
import io.mockk.Runs
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.just
import io.mockk.mockk
import io.mockk.unmockkAll
import java.time.ZonedDateTime
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.data.UUID
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.DeviceRepository
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.util.Lc
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class AccountViewModelTest {

    private val mockAccountRepository: AccountRepository = mockk(relaxUnitFun = true)
    private val mockDeviceRepository: DeviceRepository = mockk(relaxUnitFun = true)
    private val mockPaymentUseCase: PaymentUseCase = mockk(relaxed = true)

    private val dummyDevice =
        Device(
            id = DeviceId.fromString(UUID),
            name = "fake_name",
            creationDate = ZonedDateTime.now(),
        )
    private val dummyAccountNumber: AccountNumber = AccountNumber(DUMMY_DEVICE_NAME)

    private val deviceState: MutableStateFlow<DeviceState?> =
        MutableStateFlow(
            DeviceState.LoggedIn(accountNumber = dummyAccountNumber, device = dummyDevice)
        )
    private val paymentAvailability = MutableStateFlow<PaymentAvailability?>(null)
    private val accountExpiryState = MutableStateFlow(null)

    private lateinit var viewModel: AccountViewModel

    @BeforeEach
    fun setup() {
        every { mockAccountRepository.accountData } returns accountExpiryState
        every { mockDeviceRepository.deviceState } returns deviceState
        coEvery { mockPaymentUseCase.paymentAvailability } returns paymentAvailability
        coEvery { mockAccountRepository.refreshAccountData(any()) } just Runs

        viewModel =
            AccountViewModel(
                accountRepository = mockAccountRepository,
                deviceRepository = mockDeviceRepository,
                paymentUseCase = mockPaymentUseCase,
            )
    }

    @AfterEach
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun `given device state LoggedIn uiState should contain accountNumber`() = runTest {
        // Act, Assert
        viewModel.uiState.test {
            deviceState.value =
                DeviceState.LoggedIn(accountNumber = dummyAccountNumber, device = dummyDevice)
            val result = awaitItem()
            assertIs<Lc.Content<AccountUiState>>(result)
            assertEquals(dummyAccountNumber, result.value.accountNumber)
        }
    }

    @Test
    fun `onLogoutClick should invoke logout on AccountRepository`() {
        // Arrange
        coEvery { mockAccountRepository.logout() } returns Unit.right()

        // Act
        viewModel.onLogoutClick()

        // Assert
        coVerify { mockAccountRepository.logout() }
    }

    @Test
    fun `when there is a pending purchase, uiState should reflect it`() = runTest {
        // Arrange
        paymentAvailability.value =
            PaymentAvailability.ProductsAvailable(
                products =
                    listOf(
                        PaymentProduct(
                            productId = ProductId("test_product_id"),
                            price = ProductPrice("9.99"),
                            status = PaymentStatus.PENDING,
                        )
                    )
            )

        // Act, Assert
        viewModel.uiState.test {
            val result = awaitItem()
            assertIs<Lc.Content<AccountUiState>>(result)
            assertEquals(true, result.value.verificationPending)
        }
    }

    companion object {
        private const val DUMMY_DEVICE_NAME = "fake_name"
    }
}
