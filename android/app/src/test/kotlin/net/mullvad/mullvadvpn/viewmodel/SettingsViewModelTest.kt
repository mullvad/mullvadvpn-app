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
import net.mullvad.mullvadvpn.TestCoroutineRule
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.util.appVersionCallbackFlow
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class SettingsViewModelTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    private val mockDeviceRepository: DeviceRepository = mockk()
    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()

    private val mockDeviceState = MutableStateFlow<DeviceState>(DeviceState.LoggedOut)
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

    private var mockAppVersionInfoCache: AppVersionInfoCache = mockk()
    private val mockServiceConnectionContainer: ServiceConnectionContainer = mockk()

    private lateinit var viewModel: SettingsViewModel

    @Before
    fun setUp() {

        mockkStatic(CACHE_EXTENSION_CLASS)
        mockAppVersionInfoCache =
            mockk<AppVersionInfoCache>().apply {
                every { appVersionCallbackFlow() } returns versionInfo
            }

        every { mockServiceConnectionManager.connectionState } returns serviceConnectionState
        every { mockServiceConnectionContainer.appVersionInfoCache } returns mockAppVersionInfoCache
        every { mockDeviceRepository.deviceState } returns mockDeviceState
        every { mockAppVersionInfoCache.onUpdate = any() } answers {}
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
    fun test_device_state_supported_version_state() =
        runTest(testCoroutineRule.testDispatcher) {
            val updateAvailable = false
            val versionInfoTestItem =
                VersionInfo(
                    currentVersion = "1.0",
                    upgradeVersion = "1.0",
                    isOutdated = false,
                    isSupported = true
                )

            viewModel.uiState.test {
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                versionInfo.value = versionInfoTestItem
                val result = awaitItem()
                assertEquals(updateAvailable, result.isUpdateAvailable)
            }
        }

    @Test
    fun test_device_state_unsupported_version_state() = runTest {
        runTest(testCoroutineRule.testDispatcher) {
            every { mockAppVersionInfoCache.isSupported } returns false
            every { mockAppVersionInfoCache.isOutdated } returns false
            every { mockAppVersionInfoCache.version } returns ""

            viewModel.uiState.test {
                awaitItem()

                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                val result = awaitItem()
                assertEquals(true, result.isUpdateAvailable)
            }
        }
    }

    @Test
    fun test_device_state_outdated_version_state() = runTest {
        runTest(testCoroutineRule.testDispatcher) {
            every { mockAppVersionInfoCache.isSupported } returns true
            every { mockAppVersionInfoCache.isOutdated } returns true
            every { mockAppVersionInfoCache.version } returns ""

            viewModel.uiState.test {
                awaitItem()

                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                val result = awaitItem()
                assertEquals(true, result.isUpdateAvailable)
            }
        }
    }

    companion object {
        private const val CACHE_EXTENSION_CLASS = "net.mullvad.mullvadvpn.util.CacheExtensionsKt"
    }
}
