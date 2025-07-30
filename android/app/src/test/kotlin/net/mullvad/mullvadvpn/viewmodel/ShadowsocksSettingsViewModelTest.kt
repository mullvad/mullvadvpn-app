package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertIs
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.ShadowsocksSettingsUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.util.Lc
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class ShadowsocksSettingsViewModelTest {

    private val mockSettingsRepository: SettingsRepository = mockk()

    private val settingsFlow = MutableStateFlow<Settings?>(null)

    private lateinit var viewModel: ShadowsocksSettingsViewModel

    @BeforeEach
    fun setUp() {
        every { mockSettingsRepository.settingsUpdates } returns settingsFlow

        viewModel = ShadowsocksSettingsViewModel(settingsRepository = mockSettingsRepository)
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
            val result = awaitItem()
            assertIs<Lc.Content<ShadowsocksSettingsUiState>>(result)
            assertEquals(Constraint.Only(port), result.value.port)
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
            assertIs<Lc.Content<ShadowsocksSettingsUiState>>(startState)
            assertEquals(port, startState.value.customPort)

            viewModel.resetCustomPort()

            val updatedState = awaitItem()
            assertIs<Lc.Content<ShadowsocksSettingsUiState>>(updatedState)
            assertEquals(null, updatedState.value.customPort)
            coVerify { mockSettingsRepository.setCustomShadowsocksObfuscationPort(Constraint.Any) }
        }
    }
}
