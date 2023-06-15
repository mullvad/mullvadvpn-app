package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.TestCoroutineRule
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class SettingsViewModelTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    private val mockDeviceRepository: DeviceRepository = mockk()
    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()

    private val mockDeviceState = MutableStateFlow<DeviceState>(DeviceState.LoggedOut)
    private val mockConnectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Disconnected)
    private val mockAppVersionInfoCache: AppVersionInfoCache = mockk()
    private val mockServiceConnectionContainer: ServiceConnectionContainer = mockk()

    private lateinit var viewModel: SettingsViewModel

    @Before
    fun setUp() {
        every { mockServiceConnectionContainer.appVersionInfoCache } returns mockAppVersionInfoCache
        every { mockDeviceRepository.deviceState } returns mockDeviceState
        every { mockServiceConnectionManager.connectionState } returns mockConnectionState
        viewModel =
            SettingsViewModel(
                deviceRepository = mockDeviceRepository,
                serviceConnectionManager = mockServiceConnectionManager
            )
    }

    @After
    fun tearDown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun test_device_state_default_state() = runTest {
        val isLogin = false
        viewModel.uiState.test { assertEquals(isLogin, awaitItem().isLoggedIn) }
    }

    @Test
    fun test_device_state_supported_version_state() = runTest {
        val updateAvailable = false
        every { mockAppVersionInfoCache.isOutdated } returns false
        every { mockAppVersionInfoCache.isSupported } returns true
        // Act, Assert
        viewModel.uiState.test {
            mockConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            assertEquals(updateAvailable, awaitItem().isUpdateAvailable)
        }
    }
}
