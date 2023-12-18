package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.util.appVersionCallbackFlow
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class SettingsViewModelTest {

    private val mockDeviceRepository: DeviceRepository = mockk()
    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()
    private lateinit var mockAppVersionInfoCache: AppVersionInfoCache
    private val mockServiceConnectionContainer: ServiceConnectionContainer = mockk()

    private val serviceConnectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Disconnected)
    private val versionInfo =
        MutableStateFlow(
            VersionInfo(
                currentVersion = null,
                upgradeVersion = null,
                isOutdated = false,
                isSupported = false
            )
        )

    private lateinit var viewModel: SettingsViewModel

    @BeforeEach
    fun setUp() {
        mockkStatic(CACHE_EXTENSION_CLASS)
        val deviceState = MutableStateFlow<DeviceState>(DeviceState.LoggedOut)
        mockAppVersionInfoCache =
            mockk<AppVersionInfoCache>().apply {
                every { appVersionCallbackFlow() } returns versionInfo
            }

        every { mockServiceConnectionManager.connectionState } returns serviceConnectionState
        every { mockServiceConnectionContainer.appVersionInfoCache } returns mockAppVersionInfoCache
        every { mockDeviceRepository.deviceState } returns deviceState
        every { mockAppVersionInfoCache.onUpdate = any() } answers {}

        viewModel =
            SettingsViewModel(
                deviceRepository = mockDeviceRepository,
                serviceConnectionManager = mockServiceConnectionManager,
                isPlayBuild = false
            )
    }

    @AfterEach
    fun tearDown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun test_device_state_default_state() = runTest {
        // Act, Assert
        viewModel.uiState.test { assertEquals(false, awaitItem().isLoggedIn) }
    }

    @Test
    fun test_device_state_supported_version_state() = runTest {
        // Arrange
        val versionInfoTestItem =
            VersionInfo(
                currentVersion = "1.0",
                upgradeVersion = "1.0",
                isOutdated = false,
                isSupported = true
            )
        every { mockAppVersionInfoCache.version } returns "1.0"
        every { mockAppVersionInfoCache.isSupported } returns true
        every { mockAppVersionInfoCache.isOutdated } returns false

        // Act, Assert
        viewModel.uiState.test {
            awaitItem() // Wait for initial value

            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            versionInfo.value = versionInfoTestItem
            val result = awaitItem()
            assertEquals(false, result.isUpdateAvailable)
        }
    }

    @Test
    fun test_device_state_unsupported_version_state() = runTest {
        // Arrange
        every { mockAppVersionInfoCache.isSupported } returns false
        every { mockAppVersionInfoCache.isOutdated } returns false
        every { mockAppVersionInfoCache.version } returns ""

        // Act, Assert
        viewModel.uiState.test {
            awaitItem()

            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            val result = awaitItem()
            assertEquals(true, result.isUpdateAvailable)
        }
    }

    @Test
    fun test_device_state_outdated_version_state() = runTest {
        // Arrange
        every { mockAppVersionInfoCache.isSupported } returns true
        every { mockAppVersionInfoCache.isOutdated } returns true
        every { mockAppVersionInfoCache.version } returns ""

        // Act, Assert
        viewModel.uiState.test {
            awaitItem()

            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            val result = awaitItem()
            assertEquals(true, result.isUpdateAvailable)
        }
    }

    companion object {
        private const val CACHE_EXTENSION_CLASS = "net.mullvad.mullvadvpn.util.CacheExtensionsKt"
    }
}
