package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.ReceiveTurbine
import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.coEvery
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.verify
import junit.framework.Assert.assertEquals
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.TestCoroutineDispatcher
import kotlinx.coroutines.test.runBlockingTest
import kotlinx.coroutines.test.setMain
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.AccountHistory
import net.mullvad.mullvadvpn.model.DeviceListEvent
import net.mullvad.mullvadvpn.model.LoginResult
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import org.junit.Before
import org.junit.Test

class LoginViewModelTest {

    @MockK
    private lateinit var mockedAccountRepository: AccountRepository

    @MockK
    private lateinit var mockedDeviceRepository: DeviceRepository

    @MockK
    private lateinit var mockedServiceConnectionContainer: ServiceConnectionContainer

    private lateinit var loginViewModel: LoginViewModel

    private val accountCreationTestEvents = MutableSharedFlow<AccountCreationResult>()
    private val accountHistoryTestEvents = MutableStateFlow<AccountHistory>(AccountHistory.Missing)
    private val loginTestEvents = MutableSharedFlow<Event.LoginEvent>()

    private val serviceConnectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Disconnected)

    @Before
    fun setup() {
        Dispatchers.setMain(TestCoroutineDispatcher())
        MockKAnnotations.init(this, relaxUnitFun = true)

        every { mockedAccountRepository.accountCreationEvents } returns accountCreationTestEvents
        every { mockedAccountRepository.accountHistoryEvents } returns accountHistoryTestEvents
        every { mockedAccountRepository.loginEvents } returns loginTestEvents

        serviceConnectionState.value =
            ServiceConnectionState.ConnectedReady(mockedServiceConnectionContainer)

        loginViewModel = LoginViewModel(
            mockedAccountRepository,
            mockedDeviceRepository,
            TestCoroutineDispatcher()
        )
    }

    @Test
    fun testDefaultState() = runBlockingTest {
        loginViewModel.uiState.test {
            assertEquals(LoginViewModel.LoginUiState.Default, awaitItem())
        }
    }

    @Test
    fun testCreateAccount() = runBlockingTest {
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
        coEvery {
            mockedDeviceRepository.refreshAndAwaitDeviceListWithTimeout(
                any(),
                any(),
                any(),
                any()
            )
        } returns DeviceListEvent.Available(
            DUMMY_ACCOUNT_TOKEN, listOf()
        )

        loginViewModel.uiState.test {
            skipDefaultItem()
            loginViewModel.login(DUMMY_ACCOUNT_TOKEN)
            assertEquals(LoginViewModel.LoginUiState.Loading, awaitItem())
            loginTestEvents.emit(Event.LoginEvent(LoginResult.MaxDevicesReached))
            assertEquals(
                LoginViewModel.LoginUiState.TooManyDevicesError(DUMMY_ACCOUNT_TOKEN), awaitItem()
            )
        }
    }

    @Test
    fun testLoginWithRpcError() = runBlockingTest {
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
        loginViewModel.accountHistory.test {
            skipDefaultItem()
            accountHistoryTestEvents.emit(AccountHistory.Available(DUMMY_ACCOUNT_TOKEN))
            assertEquals(AccountHistory.Available(DUMMY_ACCOUNT_TOKEN), awaitItem())
        }
    }

    @Test
    fun testClearingAccountHistory() = runBlockingTest {
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
