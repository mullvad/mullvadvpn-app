package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.FlowTurbine
import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.Runs
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.invoke
import io.mockk.just
import io.mockk.mockk
import io.mockk.slot
import io.mockk.verify
import junit.framework.Assert.assertEquals
import kotlinx.coroutines.test.runBlockingTest
import net.mullvad.mullvadvpn.model.LoginResult
import net.mullvad.mullvadvpn.model.LoginStatus
import net.mullvad.mullvadvpn.ui.serviceconnection.AccountCache
import net.mullvad.talpid.util.EventNotifier
import org.junit.Before
import org.junit.Test

class LoginViewModelTest {

    @MockK
    private lateinit var mockedAccountCache: AccountCache

    @MockK
    private lateinit var mockedLoginStatusNotifier: EventNotifier<LoginStatus?>

    @MockK
    private lateinit var mockedAccountHistoryNotifier: EventNotifier<String?>

    private lateinit var loginViewModel: LoginViewModel
    private val capturedLoginStatusNotifierCallback = slot<(LoginStatus?) -> Unit>()
    private val capturedAccountHistoryNotifierCallback = slot<(String?) -> Unit>()

    @Before
    fun setup() {
        MockKAnnotations.init(this, relaxUnitFun = true)

        every {
            mockedLoginStatusNotifier.subscribe(
                any(),
                any(),
                capture(capturedLoginStatusNotifierCallback)
            )
        } just Runs

        every {
            mockedAccountHistoryNotifier.subscribe(
                any(),
                capture(capturedAccountHistoryNotifierCallback)
            )
        } just Runs

        every { mockedAccountCache.onLoginStatusChange } returns mockedLoginStatusNotifier
        every { mockedAccountCache.onAccountHistoryChange } returns mockedAccountHistoryNotifier

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
    fun testClearingViewModel() {
        loginViewModel.updateAccountCacheInstance(mockedAccountCache)
        loginViewModel.onCleared()
        verify {
            mockedLoginStatusNotifier.unsubscribe(any())
            mockedAccountHistoryNotifier.unsubscribe(any())
        }
    }

    @Test
    fun testCreateAccount() = runBlockingTest {
        loginViewModel.updateAccountCacheInstance(mockedAccountCache)
        loginViewModel.uiState.test {
            skipDefaultItem()
            loginViewModel.createAccount()
            assertEquals(LoginViewModel.LoginUiState.CreatingAccount, awaitItem())
            capturedLoginStatusNotifierCallback.captured.invoke(DummyLoginStatus.ACCOUNT_CREATED)
            assertEquals(LoginViewModel.LoginUiState.AccountCreated, awaitItem())
        }
    }

    @Test
    fun testLoginWithValidAccount() = runBlockingTest {
        loginViewModel.updateAccountCacheInstance(mockedAccountCache)
        loginViewModel.uiState.test {
            skipDefaultItem()
            loginViewModel.login("")
            assertEquals(LoginViewModel.LoginUiState.Loading, awaitItem())
            capturedLoginStatusNotifierCallback.captured.invoke(DummyLoginStatus.SUCCESSFUL_LOGIN)
            assertEquals(LoginViewModel.LoginUiState.Success(isOutOfTime = false), awaitItem())
        }
    }

    @Test
    fun testLoginWithInvalidAccount() = runBlockingTest {
        loginViewModel.updateAccountCacheInstance(mockedAccountCache)
        loginViewModel.uiState.test {
            skipDefaultItem()
            loginViewModel.login("")
            assertEquals(LoginViewModel.LoginUiState.Loading, awaitItem())
            capturedLoginStatusNotifierCallback.captured.invoke(
                DummyLoginStatus.INVALID_ACCOUNT_ERROR
            )
            assertEquals(LoginViewModel.LoginUiState.InvalidAccountError, awaitItem())
        }
    }

    @Test
    fun testLoginWithTooManyDevicesError() = runBlockingTest {
        loginViewModel.updateAccountCacheInstance(mockedAccountCache)
        loginViewModel.uiState.test {
            skipDefaultItem()
            loginViewModel.login("")
            assertEquals(LoginViewModel.LoginUiState.Loading, awaitItem())
            capturedLoginStatusNotifierCallback.captured.invoke(DummyLoginStatus.MAX_DEVICES_ERROR)
            assertEquals(LoginViewModel.LoginUiState.TooManyDevicesError, awaitItem())
        }
    }

    @Test
    fun testLoginWithRpcError() = runBlockingTest {
        loginViewModel.updateAccountCacheInstance(mockedAccountCache)
        loginViewModel.uiState.test {
            skipDefaultItem()
            loginViewModel.login("")
            assertEquals(LoginViewModel.LoginUiState.Loading, awaitItem())
            capturedLoginStatusNotifierCallback.captured.invoke(DummyLoginStatus.RPC_ERROR)
            assertEquals(LoginViewModel.LoginUiState.OtherError("RpcError"), awaitItem())
        }
    }

    @Test
    fun testLoginWithUnknownError() = runBlockingTest {
        loginViewModel.updateAccountCacheInstance(mockedAccountCache)
        loginViewModel.uiState.test {
            skipDefaultItem()
            loginViewModel.login("")
            assertEquals(LoginViewModel.LoginUiState.Loading, awaitItem())
            capturedLoginStatusNotifierCallback.captured.invoke(DummyLoginStatus.OTHER_ERROR)
            assertEquals(LoginViewModel.LoginUiState.OtherError("OtherError"), awaitItem())
        }
    }

    @Test
    fun testAccountHistory() = runBlockingTest {
        loginViewModel.updateAccountCacheInstance(mockedAccountCache)
        loginViewModel.accountHistory.test { skipDefaultItem() }
        capturedAccountHistoryNotifierCallback.invoke(DUMMY_ACCOUNT_TOKEN)
        loginViewModel.accountHistory.test { assertEquals(DUMMY_ACCOUNT_TOKEN, awaitItem()) }
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

        private object DummyLoginStatus {
            val ACCOUNT_CREATED = LoginStatus(
                DUMMY_ACCOUNT_TOKEN,
                mockk(),
                isNewAccount = true,
                mockk()
            )

            val SUCCESSFUL_LOGIN = LoginStatus(
                DUMMY_ACCOUNT_TOKEN,
                mockk(),
                isNewAccount = false,
                LoginResult.Ok
            )

            val INVALID_ACCOUNT_ERROR = LoginStatus(
                DUMMY_ACCOUNT_TOKEN,
                mockk(),
                isNewAccount = false,
                LoginResult.InvalidAccount
            )

            val MAX_DEVICES_ERROR = LoginStatus(
                DUMMY_ACCOUNT_TOKEN,
                mockk(),
                isNewAccount = false,
                LoginResult.MaxDevicesReached
            )

            val RPC_ERROR = LoginStatus(
                DUMMY_ACCOUNT_TOKEN,
                mockk(),
                isNewAccount = false,
                LoginResult.RpcError
            )

            val OTHER_ERROR = LoginStatus(
                DUMMY_ACCOUNT_TOKEN,
                mockk(),
                isNewAccount = false,
                LoginResult.OtherError
            )
        }
    }
}
