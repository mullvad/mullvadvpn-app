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
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoCache
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class SettingsViewModelTest {

    private val mockDeviceRepository: DeviceRepository = mockk()
    private val mockAppVersionInfoCache: AppVersionInfoCache = mockk()

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
    fun setup() {
        val deviceState = MutableStateFlow<DeviceState>(DeviceState.LoggedOut)

        every { mockDeviceRepository.deviceState } returns deviceState
        every { mockAppVersionInfoCache.versionInfo() } returns versionInfo

        viewModel =
            SettingsViewModel(
                deviceRepository = mockDeviceRepository,
                appVersionInfoCache = mockAppVersionInfoCache,
                isPlayBuild = false
            )
    }

    @AfterEach
    fun tearDown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `uiState should return isLoggedIn false by default`() = runTest {
        // Act, Assert
        viewModel.uiState.test { assertEquals(false, awaitItem().isLoggedIn) }
    }

    @Test
    fun `when AppVersionInfoCache returns isOutdated false uiState should return isUpdateAvailable false`() =
        runTest {
            // Arrange
            val versionInfoTestItem =
                VersionInfo(
                    currentVersion = "1.0",
                    upgradeVersion = "1.0",
                    isOutdated = false,
                    isSupported = true
                )

            // Act, Assert
            viewModel.uiState.test {
                awaitItem() // Wait for initial value

                versionInfo.value = versionInfoTestItem
                val result = awaitItem()
                assertEquals(false, result.isUpdateAvailable)
            }
        }

    @Test
    fun `when AppVersionInfoCache returns isSupported false uiState should return isUpdateAvailable true`() =
        runTest {
            // Arrange
            val versionInfoTestItem =
                VersionInfo(
                    currentVersion = "",
                    upgradeVersion = "",
                    isOutdated = false,
                    isSupported = false
                )
            versionInfo.value = versionInfoTestItem

            // Act, Assert
            viewModel.uiState.test {
                val result = awaitItem()
                assertEquals(true, result.isUpdateAvailable)
            }
        }

    @Test
    fun `when AppVersionInfoCache returns isOutdated true uiState should return isUpdateAvailable true`() =
        runTest {
            // Arrange
            val versionInfoTestItem =
                VersionInfo(
                    currentVersion = "",
                    upgradeVersion = "",
                    isOutdated = true,
                    isSupported = true
                )
            versionInfo.value = versionInfoTestItem

            // Act, Assert
            viewModel.uiState.test {
                val result = awaitItem()
                assertEquals(true, result.isUpdateAvailable)
            }
        }
}
