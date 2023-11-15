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
import kotlin.test.assertNull
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.dialog.payment.PaymentDialogData
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
import net.mullvad.mullvadvpn.util.toPaymentDialogData
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class AccountViewModelTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

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

    @Before
    fun setUp() {
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
                paymentUseCase = mockPaymentUseCase
            )
    }

    @After
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun testAccountLoggedInState() = runTest {
        // Act, Assert
        viewModel.uiState.test {
            awaitItem() // Default state
            deviceState.value = DeviceState.LoggedIn(accountAndDevice = dummyAccountAndDevice)
            val result = awaitItem()
            assertEquals(DUMMY_DEVICE_NAME, result.accountNumber)
        }
    }

    @Test
    fun testOnLogoutClick() {
        // Act
        viewModel.onLogoutClick()

        // Assert
        verify { mockAccountRepository.logout() }
    }

    @Test
    fun testBillingProductsUnavailableState() = runTest {
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
    fun testBillingProductsGenericErrorState() = runTest {
        // Act, Assert
        viewModel.uiState.test {
            awaitItem() // Default state
            paymentAvailability.tryEmit(PaymentAvailability.Error.Other(mockk()))
            val result = awaitItem().billingPaymentState
            assertIs<PaymentState.Error.Generic>(result)
        }
    }

    @Test
    fun testBillingProductsBillingErrorState() = runTest {
        // Act, Assert
        viewModel.uiState.test {
            awaitItem() // Default state
            paymentAvailability.tryEmit(PaymentAvailability.Error.BillingUnavailable)
            val result = awaitItem().billingPaymentState
            assertIs<PaymentState.Error.Billing>(result)
        }
    }

    @Test
    fun testBillingProductsPaymentAvailableState() = runTest {
        // Arrange
        val mockProduct: PaymentProduct = mockk()
        val expectedProductList = listOf(mockProduct)

        // Act, Assert
        viewModel.uiState.test {
            awaitItem() // Default state
            paymentAvailability.tryEmit(PaymentAvailability.ProductsAvailable(listOf(mockProduct)))
            val result = awaitItem().billingPaymentState
            assertIs<PaymentState.PaymentAvailable>(result)
            assertLists(expectedProductList, result.products)
        }
    }

    @Test
    fun testBillingUserCancelled() = runTest {
        // Arrange
        val result = PurchaseResult.Completed.Cancelled
        purchaseResult.value = result
        every { result.toPaymentDialogData() } returns null

        // Act, Assert
        viewModel.uiState.test { assertNull(awaitItem().paymentDialogData) }
    }

    @Test
    fun testBillingPurchaseSuccess() = runTest {
        // Arrange
        val result = PurchaseResult.Completed.Success
        val expectedData: PaymentDialogData = mockk()
        purchaseResult.value = result
        every { result.toPaymentDialogData() } returns expectedData

        // Act, Assert
        viewModel.uiState.test { assertEquals(expectedData, awaitItem().paymentDialogData) }
    }

    @Test
    fun testStartBillingPayment() {
        // Arrange
        val mockProductId = ProductId("MOCK")
        val mockActivityProvider = mockk<() -> Activity>()

        // Act
        viewModel.startBillingPayment(mockProductId, mockActivityProvider)

        // Assert
        coVerify { mockPaymentUseCase.purchaseProduct(mockProductId, mockActivityProvider) }
    }

    @Test
    fun testOnClosePurchaseResultDialogSuccessful() {
        // Arrange

        // Act
        viewModel.onClosePurchaseResultDialog(success = true)

        // Assert
        verify { mockAccountRepository.fetchAccountExpiry() }
        coVerify { mockPaymentUseCase.resetPurchaseResult() }
    }

    @Test
    fun testOnClosePurchaseResultDialogNotSuccessful() {
        // Arrange

        // Act
        viewModel.onClosePurchaseResultDialog(success = false)

        // Assert
        coVerify { mockPaymentUseCase.queryPaymentAvailability() }
        coVerify { mockPaymentUseCase.resetPurchaseResult() }
    }

    companion object {
        private const val CACHE_EXTENSION_CLASS = "net.mullvad.mullvadvpn.util.CacheExtensionsKt"
        private const val PURCHASE_RESULT_EXTENSIONS_CLASS =
            "net.mullvad.mullvadvpn.util.PurchaseResultExtensionsKt"
        private const val DUMMY_DEVICE_NAME = "fake_name"
    }
}
