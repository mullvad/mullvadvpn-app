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
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class VersionNotificationUseCaseTest {

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

    @BeforeEach
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

    @AfterEach
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun `initial state should be empty`() = runTest {
        // Arrange, Act, Assert
        versionNotificationUseCase.notifications().test { assertTrue { awaitItem().isEmpty() } }
    }

    @Test
    fun `when a new version is available use case should emit UpdateAvailable with new version`() =
        runTest {
            versionNotificationUseCase.notifications().test {
                // Arrange, Act
                val upgradeVersionInfo =
                    VersionInfo("1.0", "1.1", isOutdated = true, isSupported = true)
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                awaitItem()
                versionInfo.value = upgradeVersionInfo

                // Assert
                assertEquals(
                    awaitItem(),
                    listOf(InAppNotification.UpdateAvailable(upgradeVersionInfo))
                )
            }
        }

    @Test
    fun `when an unsupported version use case should emit UnsupportedVersion notification`() =
        runTest {
            versionNotificationUseCase.notifications().test {
                // Arrange, Act
                val upgradeVersionInfo =
                    VersionInfo("1.0", "", isOutdated = false, isSupported = false)
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
