package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.ReceiveTurbine
import app.cash.turbine.test
import app.cash.turbine.turbineScope
import arrow.core.left
import arrow.core.right
import io.mockk.MockKAnnotations
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.mockk
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
import net.mullvad.mullvadvpn.lib.account.AccountRepository
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.AccountData
import net.mullvad.mullvadvpn.model.AccountToken
import net.mullvad.mullvadvpn.model.LoginAccountError
import net.mullvad.mullvadvpn.usecase.ConnectivityUseCase
import net.mullvad.mullvadvpn.usecase.NewDeviceNotificationUseCase
import org.joda.time.DateTime
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class LoginViewModelTest {

    @MockK private lateinit var connectivityUseCase: ConnectivityUseCase
    @MockK private lateinit var mockedAccountRepository: AccountRepository
    @MockK private lateinit var mockedNewDeviceNotificationUseCase: NewDeviceNotificationUseCase

    private lateinit var loginViewModel: LoginViewModel

    @BeforeEach
    fun setup() {

        Dispatchers.setMain(UnconfinedTestDispatcher())
        MockKAnnotations.init(this, relaxUnitFun = true)
        every { connectivityUseCase.isInternetAvailable() } returns true
        every { mockedNewDeviceNotificationUseCase.newDeviceCreated() } returns Unit

        loginViewModel =
            LoginViewModel(
                accountRepository = mockedAccountRepository,
                newDeviceNotificationUseCase = mockedNewDeviceNotificationUseCase,
                connectivityUseCase = connectivityUseCase,
                UnconfinedTestDispatcher()
            )
    }

    @Test
    fun `given no internet when logging in then show no internet error`() = runTest {
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
    fun `initial state should be initial`() = runTest {
        loginViewModel.uiState.test { assertEquals(LoginUiState.INITIAL, awaitItem()) }
    }

    @Test
    fun `createAccount call should result in NavigateToWelcome side effect`() = runTest {
        turbineScope {
            // Arrange
            val uiStates = loginViewModel.uiState.testIn(backgroundScope)
            val sideEffects = loginViewModel.uiSideEffect.testIn(backgroundScope)
            coEvery { mockedAccountRepository.createAccount() } returns
                AccountToken(DUMMY_ACCOUNT_TOKEN).right()

            // Act, Assert
            uiStates.skipDefaultItem()
            loginViewModel.createAccount()
            assertEquals(Loading.CreatingAccount, uiStates.awaitItem().loginState)
            assertEquals(LoginUiSideEffect.NavigateToWelcome, sideEffects.awaitItem())
        }
    }

    @Test
    fun `given valid account when logging in then navigate to connect view`() = runTest {
        turbineScope {
            // Arrange
            val uiStates = loginViewModel.uiState.testIn(backgroundScope)
            val sideEffects = loginViewModel.uiSideEffect.testIn(backgroundScope)
            coEvery { mockedAccountRepository.login(any()) } returns Unit.right()
            coEvery { mockedAccountRepository.accountData } returns
                MutableStateFlow(AccountData(mockk(), DateTime.now().plusDays(3)))

            // Act, Assert
            uiStates.skipDefaultItem()
            loginViewModel.login(DUMMY_ACCOUNT_TOKEN)
            assertEquals(Loading.LoggingIn, uiStates.awaitItem().loginState)
            assertEquals(Success, uiStates.awaitItem().loginState)
            assertEquals(LoginUiSideEffect.NavigateToConnect, sideEffects.awaitItem())
        }
    }

    @Test
    fun `given invalid account when logging in then show invalid credentials`() = runTest {
        loginViewModel.uiState.test {
            // Arrange
            coEvery { mockedAccountRepository.login(any()) } returns
                LoginAccountError.InvalidAccount.left()

            // Act, Assert
            skipDefaultItem()
            loginViewModel.login(DUMMY_ACCOUNT_TOKEN)
            assertEquals(Loading.LoggingIn, awaitItem().loginState)
            assertEquals(Idle(loginError = LoginError.InvalidCredentials), awaitItem().loginState)
        }
    }

    @Test
    fun `given account with max devices reached when logging devices reached then navigate to too many devices`() =
        runTest {
            turbineScope {
                // Arrange
                val uiStates = loginViewModel.uiState.testIn(backgroundScope)
                val sideEffects = loginViewModel.uiSideEffect.testIn(backgroundScope)
                coEvery { mockedAccountRepository.login(any()) } returns
                    LoginAccountError.MaxDevicesReached(AccountToken(DUMMY_ACCOUNT_TOKEN)).left()

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
    fun `given RpcError when logging in then show unknown error with message`() = runTest {
        loginViewModel.uiState.test {
            // Arrange
            coEvery { mockedAccountRepository.login(any()) } returns
                LoginAccountError.RpcError.left()

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
    fun `given unknown error when logging in then show unknown error with message`() = runTest {
        loginViewModel.uiState.test {
            // Arrange
            coEvery { mockedAccountRepository.login(any()) } returns
                LoginAccountError.Unknown(mockk()).left()

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
    fun `on new accountHistory emission uiState should include lastUsedAccount matching accountHistory`() =
        runTest {
            // Arrange
            coEvery { mockedAccountRepository.fetchAccountHistory() } returns
                AccountToken(DUMMY_ACCOUNT_TOKEN)
            loginViewModel.uiState.test {
                // Act, Assert
                skipDefaultItem()
                assertEquals(
                    LoginUiState.INITIAL.copy(lastUsedAccount = AccountToken(DUMMY_ACCOUNT_TOKEN)),
                    awaitItem()
                )
            }
        }

    @Test
    fun `clearAccountHistory should invoke clearAccountHistory on AccountRepository`() = runTest {
        // Act, Assert
        loginViewModel.clearAccountHistory()
        coVerify { mockedAccountRepository.clearAccountHistory() }
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
