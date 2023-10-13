package net.mullvad.mullvadvpn.usecase

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.repository.InAppNotification
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

class VersionNotificationUseCaseTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

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
                isSupported = true
            )
        )
    private lateinit var versionNotificationUseCase: VersionNotificationUseCase

    @Before
    fun setup() {
        MockKAnnotations.init(this)
        mockkStatic(CACHE_EXTENSION_CLASS)
        mockAppVersionInfoCache =
            mockk<AppVersionInfoCache>().apply {
                every { appVersionCallbackFlow() } returns versionInfo
            }

        every { mockServiceConnectionManager.connectionState } returns serviceConnectionState
        every { mockServiceConnectionContainer.appVersionInfoCache } returns mockAppVersionInfoCache
        every { mockAppVersionInfoCache.onUpdate = any() } answers {}

        versionNotificationUseCase =
            VersionNotificationUseCase(
                serviceConnectionManager = mockServiceConnectionManager,
                isVersionInfoNotificationEnabled = true
            )
    }

    @After
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun `ensure notifications are empty by default`() = runTest {
        // Arrange, Act, Assert
        versionNotificationUseCase.notifications().test { assertTrue { awaitItem().isEmpty() } }
    }

    @Test
    fun `ensure UpdateAvailable notification is created`() = runTest {
        versionNotificationUseCase.notifications().test {
            // Arrange, Act
            val upgradeVersionInfo =
                VersionInfo("1.0", "1.1", isOutdated = true, isSupported = true)
            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            awaitItem()
            versionInfo.value = upgradeVersionInfo

            // Assert
            assertEquals(awaitItem(), listOf(InAppNotification.UpdateAvailable(upgradeVersionInfo)))
        }
    }

    @Test
    fun `ensure UnsupportedVersion notification is created`() = runTest {
        versionNotificationUseCase.notifications().test {
            // Arrange, Act
            val upgradeVersionInfo = VersionInfo("1.0", "", isOutdated = false, isSupported = false)
            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            awaitItem()
            versionInfo.value = upgradeVersionInfo

            // Assert
            assertEquals(
                awaitItem(),
                listOf(InAppNotification.UnsupportedVersion(upgradeVersionInfo))
            )
        }
    }

    companion object {
        private const val CACHE_EXTENSION_CLASS = "net.mullvad.mullvadvpn.util.CacheExtensionsKt"
    }
}
