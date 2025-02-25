package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import java.time.ZonedDateTime
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class WelcomeViewModelTest {

    private val serviceConnectionStateFlow =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Unbound)
    private val deviceStateFlow = MutableStateFlow<DeviceState?>(DeviceState.LoggedOut)
    private val accountExpiryStateFlow = MutableStateFlow<AccountData?>(null)
    private val purchaseResultFlow = MutableStateFlow<PurchaseResult?>(null)
    private val paymentAvailabilityFlow = MutableStateFlow<PaymentAvailability?>(null)

    // ConnectionProxy
    private val mockConnectionProxy: ConnectionProxy = mockk()

    // Event notifiers
    private val tunnelState = MutableStateFlow<TunnelState>(TunnelState.Disconnected())

    private val mockAccountRepository: AccountRepository = mockk(relaxed = true)
    private val mockDeviceRepository: DeviceRepository = mockk(relaxed = true)
    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()
    private val mockPaymentUseCase: PaymentUseCase = mockk(relaxed = true)

    private lateinit var viewModel: WelcomeViewModel

    @BeforeEach
    fun setup() {
        mockkStatic(PURCHASE_RESULT_EXTENSIONS_CLASS)

        every { mockDeviceRepository.deviceState } returns deviceStateFlow

        every { mockServiceConnectionManager.connectionState } returns serviceConnectionStateFlow

        every { mockConnectionProxy.tunnelState } returns tunnelState

        every { mockAccountRepository.accountData } returns accountExpiryStateFlow

        coEvery { mockPaymentUseCase.purchaseResult } returns purchaseResultFlow

        coEvery { mockPaymentUseCase.paymentAvailability } returns paymentAvailabilityFlow

        viewModel =
            WelcomeViewModel(
                accountRepository = mockAccountRepository,
                deviceRepository = mockDeviceRepository,
                paymentUseCase = mockPaymentUseCase,
                connectionProxy = mockConnectionProxy,
                pollAccountExpiry = false,
                isPlayBuild = false,
            )
    }

    @AfterEach
    fun tearDown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `on onSitePaymentClick call uiSideEffect should emit OpenAccountView`() = runTest {
        // Arrange
        val mockToken = WebsiteAuthToken.fromString("154c4cc94810fddac78398662b7fa0c7")
        coEvery { mockAccountRepository.getWebsiteAuthToken() } returns mockToken

        // Act, Assert
        viewModel.uiSideEffect.test {
            viewModel.onSitePaymentClick()
            val action = awaitItem()
            assertIs<WelcomeViewModel.UiSideEffect.OpenAccountView>(action)
            assertEquals(mockToken, action.token)
        }
    }

    @Test
    fun `on new TunnelState uiState should include new TunnelState`() = runTest {
        // Arrange
        val tunnelUiStateTestItem: TunnelState = mockk()

        // Act, Assert
        viewModel.uiState.test {
            // Default state
            awaitItem()
            tunnelState.emit(tunnelUiStateTestItem)
            val result = awaitItem()
            assertEquals(tunnelUiStateTestItem, result.tunnelState)
        }
    }

    @Test
    fun `when DeviceRepository returns LoggedIn uiState should include new accountNumber`() =
        runTest {
            // Arrange
            val expectedAccountNumber = AccountNumber("4444555566667777")
            val device: Device = mockk()
            every { device.displayName() } returns ""

            // Act, Assert
            viewModel.uiState.test {
                // Default state
                awaitItem()
                paymentAvailabilityFlow.value = null
                deviceStateFlow.value =
                    DeviceState.LoggedIn(accountNumber = expectedAccountNumber, device = device)
                assertEquals(expectedAccountNumber, awaitItem().accountNumber)
            }
        }

    @Test
    fun `when user has added time then uiSideEffect should emit OpenConnectScreen`() = runTest {
        // Arrange
        accountExpiryStateFlow.emit(
            AccountData(mockk(relaxed = true), ZonedDateTime.now().plusDays(1))
        )

        // Act, Assert
        viewModel.uiSideEffect.test {
            val action = awaitItem()
            assertIs<WelcomeViewModel.UiSideEffect.OpenConnectScreen>(action)
        }
    }

    @Test
    fun `when paymentAvailability emits ProductsUnavailable uiState should include state NoPayment`() =
        runTest {
            // Arrange
            val productsUnavailable = PaymentAvailability.ProductsUnavailable

            // Act, Assert
            viewModel.uiState.test {
                // Default item
                awaitItem()
                paymentAvailabilityFlow.tryEmit(productsUnavailable)
                val result = awaitItem().billingPaymentState
                assertIs<PaymentState.NoPayment>(result)
            }
        }

    @Test
    fun `when paymentAvailability emits ErrorOther uiState should include state ErrorGeneric`() =
        runTest {
            // Arrange
            val paymentOtherError = PaymentAvailability.Error.Other(mockk())
            paymentAvailabilityFlow.tryEmit(paymentOtherError)

            // Act, Assert
            viewModel.uiState.test {
                val result = awaitItem().billingPaymentState
                assertIs<PaymentState.Error.Generic>(result)
            }
        }

    @Test
    fun `when paymentAvailability emits ErrorBillingUnavailable uiState should include state ErrorBilling`() =
        runTest { // Arrange
            val paymentBillingError = PaymentAvailability.Error.BillingUnavailable
            paymentAvailabilityFlow.value = paymentBillingError

            // Act, Assert
            viewModel.uiState.test {
                val result = awaitItem().billingPaymentState
                assertIs<PaymentState.Error.Billing>(result)
            }
        }

    @Test
    fun `when paymentAvailability emits ProductsAvailable uiState should include state Available with products`() =
        runTest {
            // Arrange
            val mockProduct: PaymentProduct = mockk()
            val expectedProductList = listOf(mockProduct)
            val productsAvailable = PaymentAvailability.ProductsAvailable(listOf(mockProduct))
            paymentAvailabilityFlow.value = productsAvailable

            // Act, Assert
            viewModel.uiState.test {
                val result = awaitItem().billingPaymentState
                assertIs<PaymentState.PaymentAvailable>(result)
                assertLists(expectedProductList, result.products)
            }
        }

    @Test
    fun `when on disconnect click is called should call connection proxy disconnect`() = runTest {
        // Arrange
        coEvery { mockConnectionProxy.disconnect() } returns true.right()

        // Act
        viewModel.onDisconnectClick()

        // Assert
        coVerify { mockConnectionProxy.disconnect() }
    }

    companion object {
        private const val PURCHASE_RESULT_EXTENSIONS_CLASS =
            "net.mullvad.mullvadvpn.util.PurchaseResultExtensionsKt"
    }
}
