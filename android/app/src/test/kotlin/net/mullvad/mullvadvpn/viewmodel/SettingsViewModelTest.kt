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

    private lateinit var viewModel: SettingsViewModel

    @Before
    fun setUp() {
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
}
