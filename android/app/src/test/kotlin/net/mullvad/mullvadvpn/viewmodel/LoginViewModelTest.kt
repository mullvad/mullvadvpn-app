package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.ReceiveTurbine
import app.cash.turbine.test
import app.cash.turbine.turbineScope
import io.mockk.MockKAnnotations
import io.mockk.coEvery
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.verify
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import net.mullvad.mullvadvpn.compose.state.LoginError
import net.mullvad.mullvadvpn.compose.state.LoginState.Idle
import net.mullvad.mullvadvpn.compose.state.LoginState.Loading
import net.mullvad.mullvadvpn.compose.state.LoginState.Success
import net.mullvad.mullvadvpn.compose.state.LoginUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.AccountHistory
import net.mullvad.mullvadvpn.model.AccountToken
import net.mullvad.mullvadvpn.model.DeviceListEvent
import net.mullvad.mullvadvpn.model.LoginResult
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.usecase.ConnectivityUseCase
import net.mullvad.mullvadvpn.usecase.NewDeviceNotificationUseCase
import org.joda.time.DateTime
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class LoginViewModelTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    @MockK private lateinit var connectivityUseCase: ConnectivityUseCase
    @MockK private lateinit var mockedAccountRepository: AccountRepository
    @MockK private lateinit var mockedDeviceRepository: DeviceRepository
    @MockK private lateinit var mockedNewDeviceNotificationUseCase: NewDeviceNotificationUseCase

    private lateinit var loginViewModel: LoginViewModel
    private val accountHistoryTestEvents = MutableStateFlow<AccountHistory>(AccountHistory.Missing)

    @Before
    fun setup() {

        Dispatchers.setMain(UnconfinedTestDispatcher())
        MockKAnnotations.init(this, relaxUnitFun = true)
        every { connectivityUseCase.isInternetAvailable() } returns true
        every { mockedAccountRepository.accountHistory } returns accountHistoryTestEvents
        every { mockedNewDeviceNotificationUseCase.newDeviceCreated() } returns Unit

        loginViewModel =
            LoginViewModel(
                mockedAccountRepository,
                mockedDeviceRepository,
                mockedNewDeviceNotificationUseCase,
                connectivityUseCase,
                UnconfinedTestDispatcher()
            )
    }

    @Test
    fun testIsInternetAvailableWithoutInternet() = runTest {
        turbineScope {
            // Arrange
            every { connectivityUseCase.isInternetAvailable() } returns false
            val uiStates = loginViewModel.uiState.testIn(backgroundScope)

            // Act
            loginViewModel.login("")

            // Discard default item
            uiStates.awaitItem()

            // Assert
            assertEquals(
                Idle(loginError = LoginError.NoInternetConnection),
                uiStates.awaitItem().loginState
            )
        }
    }

    @Test
    fun testDefaultState() = runTest {
        loginViewModel.uiState.test { assertEquals(LoginUiState.INITIAL, awaitItem()) }
    }

    @Test
    fun testCreateAccount() = runTest {
        turbineScope {
            // Arrange
            val uiStates = loginViewModel.uiState.testIn(backgroundScope)
            val sideEffects = loginViewModel.uiSideEffect.testIn(backgroundScope)
            coEvery { mockedAccountRepository.createAccount() } returns
                AccountCreationResult.Success(DUMMY_ACCOUNT_TOKEN)

            // Act, Assert
            uiStates.skipDefaultItem()
            loginViewModel.createAccount()
            assertEquals(Loading.CreatingAccount, uiStates.awaitItem().loginState)
            assertEquals(LoginUiSideEffect.NavigateToWelcome, sideEffects.awaitItem())
        }
    }

    @Test
    fun testLoginWithValidAccount() = runTest {
        turbineScope {
            // Arrange
            val uiStates = loginViewModel.uiState.testIn(backgroundScope)
            val sideEffects = loginViewModel.uiSideEffect.testIn(backgroundScope)
            coEvery { mockedAccountRepository.login(any()) } returns LoginResult.Ok
            coEvery { mockedAccountRepository.accountExpiryState } returns
                MutableStateFlow(AccountExpiry.Available(DateTime.now().plusDays(3)))

            // Act, Assert
            uiStates.skipDefaultItem()
            loginViewModel.login(DUMMY_ACCOUNT_TOKEN)
            assertEquals(Loading.LoggingIn, uiStates.awaitItem().loginState)
            assertEquals(Success, uiStates.awaitItem().loginState)
            assertEquals(LoginUiSideEffect.NavigateToConnect, sideEffects.awaitItem())
        }
    }

    @Test
    fun testLoginWithInvalidAccount() = runTest {
        loginViewModel.uiState.test {
            // Arrange
            coEvery { mockedAccountRepository.login(any()) } returns LoginResult.InvalidAccount

            // Act, Assert
            skipDefaultItem()
            loginViewModel.login(DUMMY_ACCOUNT_TOKEN)
            assertEquals(Loading.LoggingIn, awaitItem().loginState)
            assertEquals(Idle(loginError = LoginError.InvalidCredentials), awaitItem().loginState)
        }
    }

    @Test
    fun testLoginWithTooManyDevicesError() = runTest {
        turbineScope {
            // Arrange
            val uiStates = loginViewModel.uiState.testIn(backgroundScope)
            val sideEffects = loginViewModel.uiSideEffect.testIn(backgroundScope)
            coEvery {
                mockedDeviceRepository.refreshAndAwaitDeviceListWithTimeout(
                    any(),
                    any(),
                    any(),
                    any()
                )
            } returns DeviceListEvent.Available(DUMMY_ACCOUNT_TOKEN, listOf())
            coEvery { mockedAccountRepository.login(any()) } returns LoginResult.MaxDevicesReached

            // Act, Assert
            uiStates.skipDefaultItem()
            loginViewModel.login(DUMMY_ACCOUNT_TOKEN)
            assertEquals(Loading.LoggingIn, uiStates.awaitItem().loginState)
            assertEquals(
                LoginUiSideEffect.TooManyDevices(AccountToken(DUMMY_ACCOUNT_TOKEN)),
                sideEffects.awaitItem()
            )
        }
    }

    @Test
    fun testLoginWithRpcError() = runTest {
        loginViewModel.uiState.test {
            // Arrange
            coEvery { mockedAccountRepository.login(any()) } returns LoginResult.RpcError

            // Act, Assert
            skipDefaultItem()
            loginViewModel.login(DUMMY_ACCOUNT_TOKEN)
            assertEquals(Loading.LoggingIn, awaitItem().loginState)
            assertEquals(
                Idle(LoginError.Unknown(EXPECTED_RPC_ERROR_MESSAGE)),
                awaitItem().loginState
            )
        }
    }

    @Test
    fun testLoginWithUnknownError() = runTest {
        loginViewModel.uiState.test {
            // Arrange
            coEvery { mockedAccountRepository.login(any()) } returns LoginResult.OtherError

            // Act, Assert
            skipDefaultItem()
            loginViewModel.login(DUMMY_ACCOUNT_TOKEN)
            assertEquals(Loading.LoggingIn, awaitItem().loginState)
            assertEquals(
                Idle(LoginError.Unknown(EXPECTED_OTHER_ERROR_MESSAGE)),
                awaitItem().loginState
            )
        }
    }

    @Test
    fun testAccountHistory() = runTest {
        loginViewModel.uiState.test {
            // Act, Assert
            skipDefaultItem()
            accountHistoryTestEvents.emit(AccountHistory.Available(DUMMY_ACCOUNT_TOKEN))
            assertEquals(
                LoginUiState.INITIAL.copy(lastUsedAccount = AccountToken(DUMMY_ACCOUNT_TOKEN)),
                awaitItem()
            )
        }
    }

    @Test
    fun testClearingAccountHistory() = runTest {
        // Act, Assert
        loginViewModel.clearAccountHistory()
        verify { mockedAccountRepository.clearAccountHistory() }
    }

    private suspend fun <T> ReceiveTurbine<T>.skipDefaultItem() where T : Any? {
        awaitItem()
    }

    companion object {
        private const val DUMMY_ACCOUNT_TOKEN = "DUMMY"
        private const val EXPECTED_RPC_ERROR_MESSAGE = "RpcError"
        private const val EXPECTED_OTHER_ERROR_MESSAGE = "OtherError"
    }
}
