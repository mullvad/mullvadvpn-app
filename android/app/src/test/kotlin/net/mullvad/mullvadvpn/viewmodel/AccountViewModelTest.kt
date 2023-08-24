package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import io.mockk.verify
import kotlin.test.assertEquals
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
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
    private val mockAuthTokenCache: AuthTokenCache = mockk()

    private val deviceState: MutableStateFlow<DeviceState> = MutableStateFlow(DeviceState.Initial)
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
        unmockkAll()
    }

    @Test
    fun testAccountLoggedInState() = runTest {
        // Act, Assert
        viewModel.uiState.test {
            var result = awaitItem()
            assertEquals(null, result.deviceName)
            deviceState.value = DeviceState.LoggedIn(accountAndDevice = dummyAccountAndDevice)
            result = awaitItem()
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

    companion object {
        private const val CACHE_EXTENSION_CLASS = "net.mullvad.mullvadvpn.util.CacheExtensionsKt"
        private const val DUMMY_DEVICE_NAME = "fake_name"
    }
}
