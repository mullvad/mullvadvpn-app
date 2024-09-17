package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class ShadowsocksSettingsViewModelTest {

    private val mockSettingsRepository: SettingsRepository = mockk()
    private val mockRelayListRepository: RelayListRepository = mockk()

    private val settingsFlow = MutableStateFlow<Settings?>(null)
    private val portRangesFlow = MutableStateFlow<List<PortRange>>(emptyList())

    private lateinit var viewModel: ShadowsocksSettingsViewModel

    @BeforeEach
    fun setUp() {
        every { mockSettingsRepository.settingsUpdates } returns settingsFlow
        every { mockRelayListRepository.shadowsocksPortRanges } returns portRangesFlow

        viewModel =
            ShadowsocksSettingsViewModel(
                settingsRepository = mockSettingsRepository,
                relayListRepository = mockRelayListRepository,
            )
    }

    @Test
    fun `uiState should reflect latest port value from settings`() = runTest {
        // Arrange
        val mockSettings: Settings = mockk()
        val port = Port(123)
        every { mockSettings.obfuscationSettings.shadowsocks.port } returns Constraint.Only(port)

        settingsFlow.update { mockSettings }

        // Act, Assert
        viewModel.uiState.test {
            // Check result
            val result = awaitItem().port
            assertEquals(Constraint.Only(port), result)
        }
    }

    @Test
    fun `uiState should reflect latest port range value from relay list`() = runTest {
        // Arrange
        val mockSettings: Settings = mockk()
        val port = Port(123)
        every { mockSettings.obfuscationSettings.shadowsocks.port } returns Constraint.Only(port)
        val mockPortRange: List<PortRange> = listOf(mockk())

        portRangesFlow.update { mockPortRange }
        settingsFlow.update { mockSettings }

        // Act, Assert
        viewModel.uiState.test {
            // Check result
            val result = awaitItem().validPortRanges
            assertLists(mockPortRange, result)
        }
    }

    @Test
    fun `when onObfuscationPortSelected is called should call repository`() {
        // Arrange
        val port = Constraint.Only(Port(123))
        coEvery { mockSettingsRepository.setCustomShadowsocksObfuscationPort(port) } returns
            Unit.right()

        // Act
        viewModel.onObfuscationPortSelected(port)

        // Assert
        coVerify { mockSettingsRepository.setCustomShadowsocksObfuscationPort(port) }
    }

    @Test
    fun `when reset custom port is called should reset custom port`() = runTest {
        // Arrange
        val mockSettings: Settings = mockk()
        val port = Port(123) // Needs to be not in SHADOWSOCKS_PRESET_PORTS
        every { mockSettings.obfuscationSettings.shadowsocks.port } returns Constraint.Only(port)
        coEvery {
            mockSettingsRepository.setCustomShadowsocksObfuscationPort(Constraint.Any)
        } returns Unit.right()

        settingsFlow.update { mockSettings }

        // Act, Assert
        viewModel.uiState.test {
            val startState = awaitItem()
            assertEquals(port, startState.customPort)

            viewModel.resetCustomPort()

            val updatedState = awaitItem()
            assertEquals(null, updatedState.customPort)
            coVerify { mockSettingsRepository.setCustomShadowsocksObfuscationPort(Constraint.Any) }
        }
    }
}
