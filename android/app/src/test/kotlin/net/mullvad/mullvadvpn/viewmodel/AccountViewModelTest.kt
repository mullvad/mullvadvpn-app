package net.mullvad.mullvadvpn.viewmodel

import android.app.Activity
import app.cash.turbine.test
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import io.mockk.verify
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.model.AccountAndDevice
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.AuthTokenCache
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class AccountViewModelTest {

    private val mockAccountRepository: AccountRepository = mockk(relaxUnitFun = true)
    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()
    private val mockDeviceRepository: DeviceRepository = mockk()
    private val mockAuthTokenCache: AuthTokenCache = mockk()
    private val mockPaymentUseCase: PaymentUseCase = mockk(relaxed = true)

    private val deviceState: MutableStateFlow<DeviceState> = MutableStateFlow(DeviceState.Initial)
    private val paymentAvailability = MutableStateFlow<PaymentAvailability?>(null)
    private val purchaseResult = MutableStateFlow<PurchaseResult?>(null)
    private val accountExpiryState = MutableStateFlow(AccountExpiry.Missing)

    private val dummyAccountAndDevice: AccountAndDevice =
        AccountAndDevice(
            DUMMY_DEVICE_NAME,
            Device(
                id = "fake_id",
                name = "fake_name",
                pubkey = byteArrayOf(),
                created = "mock_date"
            )
        )

    private lateinit var viewModel: AccountViewModel

    @BeforeEach
    fun setup() {
        mockkStatic(CACHE_EXTENSION_CLASS)
        mockkStatic(PURCHASE_RESULT_EXTENSIONS_CLASS)
        every { mockServiceConnectionManager.authTokenCache() } returns mockAuthTokenCache
        every { mockDeviceRepository.deviceState } returns deviceState
        every { mockAccountRepository.accountExpiryState } returns accountExpiryState
        coEvery { mockPaymentUseCase.purchaseResult } returns purchaseResult
        coEvery { mockPaymentUseCase.paymentAvailability } returns paymentAvailability

        viewModel =
            AccountViewModel(
                accountRepository = mockAccountRepository,
                serviceConnectionManager = mockServiceConnectionManager,
                deviceRepository = mockDeviceRepository,
                paymentUseCase = mockPaymentUseCase,
                isPlayBuild = false
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
            awaitItem() // Default state
            deviceState.value = DeviceState.LoggedIn(accountAndDevice = dummyAccountAndDevice)
            val result = awaitItem()
            assertEquals(DUMMY_DEVICE_NAME, result.accountNumber)
        }
    }

    @Test
    fun `onLogoutClick should invoke logout on AccountRepository`() {
        // Act
        viewModel.onLogoutClick()

        // Assert
        verify { mockAccountRepository.logout() }
    }

    @Test
    fun `when paymentAvailability emits ProductsUnavailable uiState should be NoPayment`() =
        runTest {
            // Arrange in setup

            // Act, Assert
            viewModel.uiState.test {
                awaitItem() // Default state
                paymentAvailability.tryEmit(PaymentAvailability.ProductsUnavailable)
                val result = awaitItem().billingPaymentState
                assertIs<PaymentState.NoPayment>(result)
            }
        }

    @Test
    fun `when paymentAvailability emits ErrorOther uiState should be ErrorGeneric`() = runTest {
        // Act, Assert
        viewModel.uiState.test {
            awaitItem() // Default state
            paymentAvailability.tryEmit(PaymentAvailability.Error.Other(mockk()))
            val result = awaitItem().billingPaymentState
            assertIs<PaymentState.Error.Generic>(result)
        }
    }

    @Test
    fun `when paymentAvailability emits ErrorBillingUnavailable uiState should be ErrorBilling`() =
        runTest {
            // Act, Assert
            viewModel.uiState.test {
                awaitItem() // Default state
                paymentAvailability.tryEmit(PaymentAvailability.Error.BillingUnavailable)
                val result = awaitItem().billingPaymentState
                assertIs<PaymentState.Error.Billing>(result)
            }
        }

    @Test
    fun `when paymentAvailability emits ProductsAvailable uiState should be Available with products`() =
        runTest {
            // Arrange
            val mockProduct: PaymentProduct = mockk()
            val expectedProductList = listOf(mockProduct)

            // Act, Assert
            viewModel.uiState.test {
                awaitItem() // Default state
                paymentAvailability.tryEmit(
                    PaymentAvailability.ProductsAvailable(listOf(mockProduct))
                )
                val result = awaitItem().billingPaymentState
                assertIs<PaymentState.PaymentAvailable>(result)
                assertLists(expectedProductList, result.products)
            }
        }

    @Test
    fun `startBillingPayment should invoke purchaseProduct on PaymentUseCase`() {
        // Arrange
        val mockProductId = ProductId("MOCK")
        val mockActivityProvider = mockk<() -> Activity>()

        // Act
        viewModel.startBillingPayment(mockProductId, mockActivityProvider)

        // Assert
        coVerify { mockPaymentUseCase.purchaseProduct(mockProductId, mockActivityProvider) }
    }

    @Test
    fun `onClosePurchaseResultDialog with success should invoke fetchAccountExpiry on AccountRepository`() {
        // Arrange

        // Act
        viewModel.onClosePurchaseResultDialog(success = true)

        // Assert
        verify { mockAccountRepository.fetchAccountExpiry() }
    }

    @Test
    fun `onClosePurchaseResultDialog with success should invoke resetPurchaseResult on PaymentUseCase`() {
        // Arrange

        // Act
        viewModel.onClosePurchaseResultDialog(success = true)

        // Assert
        coVerify { mockPaymentUseCase.resetPurchaseResult() }
    }

    @Test
    fun `onClosePurchaseResultDialog with success false should invoke queryPaymentAvailability on PaymentUseCase`() {
        // Arrange

        // Act
        viewModel.onClosePurchaseResultDialog(success = false)

        // Assert
        coVerify { mockPaymentUseCase.queryPaymentAvailability() }
    }

    @Test
    fun `onClosePurchaseResultDialog with success false should invoke resetPurchaseResult on PaymentUseCase`() {
        // Arrange

        // Act
        viewModel.onClosePurchaseResultDialog(success = false)

        // Assert
        coVerify { mockPaymentUseCase.resetPurchaseResult() }
    }

    companion object {
        private const val CACHE_EXTENSION_CLASS = "net.mullvad.mullvadvpn.util.CacheExtensionsKt"
        private const val PURCHASE_RESULT_EXTENSIONS_CLASS =
            "net.mullvad.mullvadvpn.util.PurchaseResultExtensionsKt"
        private const val DUMMY_DEVICE_NAME = "fake_name"
    }
}
