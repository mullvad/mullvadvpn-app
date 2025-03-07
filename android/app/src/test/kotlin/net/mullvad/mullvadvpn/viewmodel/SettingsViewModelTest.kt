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
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository
import net.mullvad.mullvadvpn.lib.shared.VersionInfo
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoRepository
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class SettingsViewModelTest {

    private val mockDeviceRepository: DeviceRepository = mockk()
    private val mockAppVersionInfoRepository: AppVersionInfoRepository = mockk()
    private val mockWireguardConstraintsRepository: WireguardConstraintsRepository = mockk()
    private val mockSettingsRepository: SettingsRepository = mockk()

    private val versionInfo =
        MutableStateFlow(VersionInfo(currentVersion = "", isSupported = false))
    private val wireguardConstraints = MutableStateFlow<WireguardConstraints>(mockk(relaxed = true))
    private val settings = MutableStateFlow(mockk<Settings>(relaxed = true))

    private lateinit var viewModel: SettingsViewModel

    @BeforeEach
    fun setup() {
        val deviceState = MutableStateFlow<DeviceState>(DeviceState.LoggedOut)

        every { mockDeviceRepository.deviceState } returns deviceState
        every { mockAppVersionInfoRepository.versionInfo } returns versionInfo
        every { mockWireguardConstraintsRepository.wireguardConstraints } returns
            wireguardConstraints
        every { mockSettingsRepository.settingsUpdates } returns settings

        viewModel =
            SettingsViewModel(
                deviceRepository = mockDeviceRepository,
                appVersionInfoRepository = mockAppVersionInfoRepository,
                wireguardConstraintsRepository = mockWireguardConstraintsRepository,
                settingsRepository = mockSettingsRepository,
                isPlayBuild = false,
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
    fun `when AppVersionInfoRepository returns isSupported true uiState should return isSupportedVersion true`() =
        runTest {
            // Arrange
            val versionInfoTestItem = VersionInfo(currentVersion = "", isSupported = true)
            versionInfo.value = versionInfoTestItem

            // Act, Assert
            viewModel.uiState.test {
                val result = awaitItem()
                assertEquals(true, result.isSupportedVersion)
            }
        }

    @Test
    fun `when AppVersionInfoRepository returns isSupported false uiState should return isSupportedVersion false`() =
        runTest {
            // Arrange
            val versionInfoTestItem = VersionInfo(currentVersion = "", isSupported = false)
            versionInfo.value = versionInfoTestItem

            // Act, Assert
            viewModel.uiState.test {
                val result = awaitItem()
                assertEquals(false, result.isSupportedVersion)
            }
        }

    @Test
    fun `when WireguardConstraintsRepository return multihop enabled uiState should return multihop enabled true`() =
        runTest {
            // Arrange
            wireguardConstraints.value =
                WireguardConstraints(
                    isMultihopEnabled = true,
                    entryLocation = Constraint.Any,
                    port = Constraint.Any,
                )

            // Act, Assert
            viewModel.uiState.test {
                val result = awaitItem()
                assertEquals(true, result.multihopEnabled)
            }
        }
}
