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
import net.mullvad.mullvadvpn.compose.state.LoginState.*
import net.mullvad.mullvadvpn.compose.state.LoginUiState
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.AccountHistory
import net.mullvad.mullvadvpn.model.AccountToken
import net.mullvad.mullvadvpn.model.DeviceListEvent
import net.mullvad.mullvadvpn.model.LoginResult
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test

class LoginViewModelTest {
    @MockK private lateinit var mockedAccountRepository: AccountRepository
    @MockK private lateinit var mockedDeviceRepository: DeviceRepository

    private lateinit var loginViewModel: LoginViewModel
    private val accountHistoryTestEvents = MutableStateFlow<AccountHistory>(AccountHistory.Missing)

    @Before
    fun setup() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        MockKAnnotations.init(this, relaxUnitFun = true)

        every { mockedAccountRepository.accountHistory } returns accountHistoryTestEvents

        loginViewModel =
            LoginViewModel(
                mockedAccountRepository,
                mockedDeviceRepository,
                UnconfinedTestDispatcher()
            )
    }

    @Test
    fun testDefaultState() =
        runTest(UnconfinedTestDispatcher()) {
            loginViewModel.uiState.test { assertEquals(LoginUiState.INITIAL, awaitItem()) }
        }

    @Test
    fun testCreateAccount() =
        runTest(UnconfinedTestDispatcher()) {
            turbineScope {
                val uiStates = loginViewModel.uiState.testIn(backgroundScope)
                val sideEffects = loginViewModel.sideEffect.testIn(backgroundScope)

                uiStates.skipDefaultItem()

                coEvery { mockedAccountRepository.createAccount() } returns
                    AccountCreationResult.Success(DUMMY_ACCOUNT_TOKEN)
                loginViewModel.createAccount()
                assertEquals(Loading.CreatingAccount, uiStates.awaitItem().loginState)
                assertEquals(LoginSideEffect.NavigateToWelcome, sideEffects.awaitItem())
            }
        }

    @Test
    fun testLoginWithValidAccount() =
        runTest(UnconfinedTestDispatcher()) {
            turbineScope {
                coEvery { mockedAccountRepository.login(any()) } returns LoginResult.Ok

                val uiStates = loginViewModel.uiState.testIn(backgroundScope)
                val sideEffects = loginViewModel.sideEffect.testIn(backgroundScope)

                uiStates.skipDefaultItem()

                loginViewModel.login(DUMMY_ACCOUNT_TOKEN)
                assertEquals(Loading.LoggingIn, uiStates.awaitItem().loginState)
                assertEquals(Success, uiStates.awaitItem().loginState)
                assertEquals(LoginSideEffect.NavigateToConnect, sideEffects.awaitItem())
            }
        }

    @Test
    fun testLoginWithInvalidAccount() =
        runTest(UnconfinedTestDispatcher()) {
            coEvery { mockedAccountRepository.login(any()) } returns LoginResult.InvalidAccount

            loginViewModel.uiState.test {
                skipDefaultItem()

                loginViewModel.login(DUMMY_ACCOUNT_TOKEN)
                assertEquals(Loading.LoggingIn, awaitItem().loginState)

                assertEquals(
                    Idle(loginError = LoginError.InvalidCredentials),
                    awaitItem().loginState
                )
            }
        }

    @Test
    fun testLoginWithTooManyDevicesError() =
        runTest(UnconfinedTestDispatcher()) {
            coEvery {
                mockedDeviceRepository.refreshAndAwaitDeviceListWithTimeout(
                    any(),
                    any(),
                    any(),
                    any()
                )
            } returns DeviceListEvent.Available(DUMMY_ACCOUNT_TOKEN, listOf())

            turbineScope {
                val uiStates = loginViewModel.uiState.testIn(backgroundScope)
                val sideEffects = loginViewModel.sideEffect.testIn(backgroundScope)

                uiStates.skipDefaultItem()

                coEvery { mockedAccountRepository.login(any()) } returns
                    LoginResult.MaxDevicesReached
                loginViewModel.login(DUMMY_ACCOUNT_TOKEN)
                assertEquals(Loading.LoggingIn, uiStates.awaitItem().loginState)
                assertEquals(
                    LoginSideEffect.TooManyDevices(AccountToken(DUMMY_ACCOUNT_TOKEN)),
                    sideEffects.awaitItem()
                )
            }
        }

    @Test
    fun testLoginWithRpcError() =
        runTest(UnconfinedTestDispatcher()) {
            loginViewModel.uiState.test {
                skipDefaultItem()

                coEvery { mockedAccountRepository.login(any()) } returns LoginResult.RpcError
                loginViewModel.login(DUMMY_ACCOUNT_TOKEN)
                assertEquals(Loading.LoggingIn, awaitItem().loginState)
                assertEquals(
                    Idle(LoginError.Unknown(EXPECTED_RPC_ERROR_MESSAGE)),
                    awaitItem().loginState
                )
            }
        }

    @Test
    fun testLoginWithUnknownError() =
        runTest(UnconfinedTestDispatcher()) {
            loginViewModel.uiState.test {
                skipDefaultItem()

                coEvery { mockedAccountRepository.login(any()) } returns LoginResult.OtherError
                loginViewModel.login(DUMMY_ACCOUNT_TOKEN)
                assertEquals(Loading.LoggingIn, awaitItem().loginState)
                assertEquals(
                    Idle(LoginError.Unknown(EXPECTED_OTHER_ERROR_MESSAGE)),
                    awaitItem().loginState
                )
            }
        }

    @Test
    fun testAccountHistory() =
        runTest(UnconfinedTestDispatcher()) {
            loginViewModel.uiState.test {
                skipDefaultItem()
                accountHistoryTestEvents.emit(AccountHistory.Available(DUMMY_ACCOUNT_TOKEN))
                assertEquals(
                    LoginUiState.INITIAL.copy(lastUsedAccount = AccountToken(DUMMY_ACCOUNT_TOKEN)),
                    awaitItem()
                )
            }
        }

    @Test
    fun testClearingAccountHistory() =
        runTest(UnconfinedTestDispatcher()) {
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
