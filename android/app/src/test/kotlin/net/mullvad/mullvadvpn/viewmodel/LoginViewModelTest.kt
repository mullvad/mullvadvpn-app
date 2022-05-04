package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.FlowTurbine
import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.mockk
import io.mockk.verify
import junit.framework.Assert.assertEquals
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.test.TestCoroutineDispatcher
import kotlinx.coroutines.test.runBlockingTest
import kotlinx.coroutines.test.setMain
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.AccountHistory
import net.mullvad.mullvadvpn.model.LoginResult
import net.mullvad.mullvadvpn.ui.serviceconnection.AccountCache
import org.junit.Before
import org.junit.Test

class LoginViewModelTest {

    @MockK
    private lateinit var mockedAccountCache: AccountCache

    private lateinit var loginViewModel: LoginViewModel

    private val accountCreationTestEvents = MutableSharedFlow<AccountCreationResult>()
    private val accountHistoryTestEvents = MutableSharedFlow<AccountHistory>()
    private val loginTestEvents = MutableSharedFlow<Event.LoginEvent>()

    @Before
    fun setup() {
        Dispatchers.setMain(TestCoroutineDispatcher())
        MockKAnnotations.init(this, relaxUnitFun = true)

        every { mockedAccountCache.accountCreationEvents } returns accountCreationTestEvents
        every { mockedAccountCache.accountHistoryEvents } returns accountHistoryTestEvents
        every { mockedAccountCache.loginEvents } returns loginTestEvents

        loginViewModel = LoginViewModel(mockk())
    }

    @Test
    fun testDefaultState() = runBlockingTest {
        loginViewModel.updateAccountCacheInstance(mockedAccountCache)
        loginViewModel.uiState.test {
            assertEquals(LoginViewModel.LoginUiState.Default, awaitItem())
        }
    }

    @Test
    fun testCreateAccount() = runBlockingTest {
        loginViewModel.updateAccountCacheInstance(mockedAccountCache)
        loginViewModel.uiState.test {
            skipDefaultItem()
            loginViewModel.createAccount()
            assertEquals(LoginViewModel.LoginUiState.CreatingAccount, awaitItem())
            accountCreationTestEvents.emit(AccountCreationResult.Success(DUMMY_ACCOUNT_TOKEN))
            assertEquals(LoginViewModel.LoginUiState.AccountCreated, awaitItem())
        }
    }

    @Test
    fun testLoginWithValidAccount() = runBlockingTest {
        loginViewModel.updateAccountCacheInstance(mockedAccountCache)
        loginViewModel.uiState.test {
            skipDefaultItem()
            loginViewModel.login(DUMMY_ACCOUNT_TOKEN)
            assertEquals(LoginViewModel.LoginUiState.Loading, awaitItem())
            loginTestEvents.emit(Event.LoginEvent(LoginResult.Ok))
            assertEquals(LoginViewModel.LoginUiState.Success(isOutOfTime = false), awaitItem())
        }
    }

    @Test
    fun testLoginWithInvalidAccount() = runBlockingTest {
        loginViewModel.updateAccountCacheInstance(mockedAccountCache)
        loginViewModel.uiState.test {
            skipDefaultItem()
            loginViewModel.login(DUMMY_ACCOUNT_TOKEN)
            assertEquals(LoginViewModel.LoginUiState.Loading, awaitItem())
            loginTestEvents.emit(Event.LoginEvent(LoginResult.InvalidAccount))
            assertEquals(LoginViewModel.LoginUiState.InvalidAccountError, awaitItem())
        }
    }

    @Test
    fun testLoginWithTooManyDevicesError() = runBlockingTest {
        loginViewModel.updateAccountCacheInstance(mockedAccountCache)
        loginViewModel.uiState.test {
            skipDefaultItem()
            loginViewModel.login(DUMMY_ACCOUNT_TOKEN)
            assertEquals(LoginViewModel.LoginUiState.Loading, awaitItem())
            loginTestEvents.emit(Event.LoginEvent(LoginResult.MaxDevicesReached))
            assertEquals(LoginViewModel.LoginUiState.TooManyDevicesError, awaitItem())
        }
    }

    @Test
    fun testLoginWithRpcError() = runBlockingTest {
        loginViewModel.updateAccountCacheInstance(mockedAccountCache)
        loginViewModel.uiState.test {
            skipDefaultItem()
            loginViewModel.login(DUMMY_ACCOUNT_TOKEN)
            assertEquals(LoginViewModel.LoginUiState.Loading, awaitItem())
            loginTestEvents.emit(Event.LoginEvent(LoginResult.RpcError))
            assertEquals(
                LoginViewModel.LoginUiState.OtherError(EXPECTED_RPC_ERROR_MESSAGE),
                awaitItem()
            )
        }
    }

    @Test
    fun testLoginWithUnknownError() = runBlockingTest {
        loginViewModel.updateAccountCacheInstance(mockedAccountCache)
        loginViewModel.uiState.test {
            skipDefaultItem()
            loginViewModel.login(DUMMY_ACCOUNT_TOKEN)
            assertEquals(LoginViewModel.LoginUiState.Loading, awaitItem())
            loginTestEvents.emit(Event.LoginEvent(LoginResult.OtherError))
            assertEquals(
                LoginViewModel.LoginUiState.OtherError(EXPECTED_OTHER_ERROR_MESSAGE),
                awaitItem()
            )
        }
    }

    @Test
    fun testAccountHistory() = runBlockingTest {
        loginViewModel.updateAccountCacheInstance(mockedAccountCache)
        loginViewModel.accountHistory.test { skipDefaultItem() }
        accountHistoryTestEvents.emit(AccountHistory.Available(DUMMY_ACCOUNT_TOKEN))
        loginViewModel.accountHistory.test {
            assertEquals(AccountHistory.Available(DUMMY_ACCOUNT_TOKEN), awaitItem())
        }
    }

    @Test
    fun testClearingAccountHistory() = runBlockingTest {
        loginViewModel.updateAccountCacheInstance(mockedAccountCache)
        loginViewModel.clearAccountHistory()
        verify { mockedAccountCache.clearAccountHistory() }
    }

    private suspend fun <T> FlowTurbine<T>.skipDefaultItem() where T : Any? {
        awaitItem()
    }

    companion object {
        private const val DUMMY_ACCOUNT_TOKEN = "DUMMY"
        private const val EXPECTED_RPC_ERROR_MESSAGE = "RpcError"
        private const val EXPECTED_OTHER_ERROR_MESSAGE = "OtherError"
    }
}
