package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import io.mockk.verify
import kotlin.test.assertEquals
import kotlin.test.assertNotEquals
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.TestCoroutineRule
import net.mullvad.mullvadvpn.model.AccountAndDevice
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.AuthTokenCache
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class AccountViewModelTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    private val mockAccountRepository: AccountRepository = mockk(relaxUnitFun = true)
    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()
    private val mockDeviceRepository: DeviceRepository = mockk()

    private val deviceState: MutableStateFlow<DeviceState> = MutableStateFlow(DeviceState.Initial)
    private val accountExpiryState = MutableStateFlow(AccountExpiry.Missing)

    private val mockAuthTokenCache: AuthTokenCache = mockk()
    private val mockAccountAndDevice: AccountAndDevice =
        AccountAndDevice(
            "1111222233334444",
            Device(
                id = "fake_id",
                name = "fake_name",
                pubkey = ByteArray(1234),
                ports = ArrayList(),
                created = "mock_date"
            )
        )

    private lateinit var viewModel: AccountViewModel

    @Before
    fun setUp() {
        mockkStatic(CACHE_EXTENSION_CLASS)
        every { mockServiceConnectionManager.authTokenCache() } returns mockAuthTokenCache
        every { mockDeviceRepository.deviceState } returns deviceState
        every { mockAccountRepository.accountExpiryState } returns accountExpiryState

        viewModel =
            AccountViewModel(
                accountRepository = mockAccountRepository,
                serviceConnectionManager = mockServiceConnectionManager,
                deviceRepository = mockDeviceRepository
            )
    }

    @After
    fun tearDown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun test_account_log_in_state() =
        runTest(testCoroutineRule.testDispatcher) {
            // Act, Assert
            viewModel.uiState.test {
                var result = awaitItem()
                assertNotEquals("1111222233334444", result.accountNumber)
                deviceState.value = DeviceState.LoggedIn(accountAndDevice = mockAccountAndDevice)
                accountExpiryState.value = AccountExpiry.Missing
                result = awaitItem()
                assertEquals("1111222233334444", result.accountNumber)
            }
        }

    @Test
    fun testOnLogoutClick() {

        // Act
        viewModel.onLogoutClick()

        // Assert
        verify { mockAccountRepository.logout() }
    }

    companion object {
        private const val CACHE_EXTENSION_CLASS = "net.mullvad.mullvadvpn.util.CacheExtensionsKt"
    }
}
