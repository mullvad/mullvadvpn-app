package net.mullvad.mullvadvpn.viewmodel

import android.content.res.Resources
import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import io.mockk.verify
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.Port
import net.mullvad.mullvadvpn.model.PortRange
import net.mullvad.mullvadvpn.model.QuantumResistantState
import net.mullvad.mullvadvpn.model.RelayConstraints
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.model.TunnelOptions
import net.mullvad.mullvadvpn.model.WireguardConstraints
import net.mullvad.mullvadvpn.model.WireguardTunnelOptions
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.usecase.PortRangeUseCase
import net.mullvad.mullvadvpn.usecase.RelayListUseCase
import net.mullvad.mullvadvpn.usecase.SystemVpnSettingsUseCase
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class VpnSettingsViewModelTest {

    private val mockSettingsRepository: SettingsRepository = mockk()
    private val mockResources: Resources = mockk()
    private val mockPortRangeUseCase: PortRangeUseCase = mockk()
    private val mockRelayListUseCase: RelayListUseCase = mockk()
    private val mockSystemVpnSettingsUseCase: SystemVpnSettingsUseCase = mockk(relaxed = true)

    private val mockSettingsUpdate = MutableStateFlow<Settings?>(null)
    private val portRangeFlow = MutableStateFlow(emptyList<PortRange>())

    private lateinit var viewModel: VpnSettingsViewModel

    @BeforeEach
    fun setUp() {
        every { mockSettingsRepository.settingsUpdates } returns mockSettingsUpdate
        every { mockPortRangeUseCase.portRanges() } returns portRangeFlow

        viewModel =
            VpnSettingsViewModel(
                repository = mockSettingsRepository,
                resources = mockResources,
                portRangeUseCase = mockPortRangeUseCase,
                relayListUseCase = mockRelayListUseCase,
                systemVpnSettingsUseCase = mockSystemVpnSettingsUseCase,
                dispatcher = UnconfinedTestDispatcher()
            )
    }

    @AfterEach
    fun tearDown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun test_select_quantum_resistant_state_select() = runTest {
        val quantumResistantState = QuantumResistantState.On
        every { mockSettingsRepository.setWireguardQuantumResistant(quantumResistantState) } returns
            Unit
        viewModel.onSelectQuantumResistanceSetting(quantumResistantState)
        verify(exactly = 1) {
            mockSettingsRepository.setWireguardQuantumResistant(quantumResistantState)
        }
    }

    @Test
    fun test_update_quantum_resistant_default_state() = runTest {
        // Arrange
        val expectedResistantState = QuantumResistantState.Off

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(expectedResistantState, awaitItem().quantumResistant)
        }
    }

    @Test
    fun test_update_quantum_resistant_update_state() = runTest {
        val defaultResistantState = QuantumResistantState.Off
        val expectedResistantState = QuantumResistantState.On
        val mockSettings: Settings = mockk(relaxed = true)
        val mockTunnelOptions: TunnelOptions = mockk(relaxed = true)
        val mockWireguardTunnelOptions: WireguardTunnelOptions = mockk(relaxed = true)

        every { mockSettings.tunnelOptions } returns mockTunnelOptions
        every { mockTunnelOptions.wireguard } returns mockWireguardTunnelOptions
        every { mockWireguardTunnelOptions.quantumResistant } returns expectedResistantState
        every { mockSettings.relaySettings } returns mockk<RelaySettings.Normal>(relaxed = true)

        viewModel.uiState.test {
            assertEquals(defaultResistantState, awaitItem().quantumResistant)
            mockSettingsUpdate.value = mockSettings
            assertEquals(expectedResistantState, awaitItem().quantumResistant)
        }
    }

    @Test
    fun test_update_wireguard_port_state() = runTest {
        // Arrange
        val expectedPort: Constraint<Port> = Constraint.Only(Port(99))
        val mockSettings: Settings = mockk(relaxed = true)
        val mockRelaySettings: RelaySettings.Normal = mockk()
        val mockRelayConstraints: RelayConstraints = mockk()
        val mockWireguardConstraints: WireguardConstraints = mockk()

        every { mockSettings.relaySettings } returns mockRelaySettings
        every { mockRelaySettings.relayConstraints } returns mockRelayConstraints
        every { mockRelayConstraints.wireguardConstraints } returns mockWireguardConstraints
        every { mockWireguardConstraints.port } returns expectedPort

        // Act, Assert
        viewModel.uiState.test {
            assertIs<Constraint.Any<Port>>(awaitItem().selectedWireguardPort)
            mockSettingsUpdate.value = mockSettings
            assertEquals(expectedPort, awaitItem().customWireguardPort)
            assertEquals(expectedPort, awaitItem().selectedWireguardPort)
        }
    }

    @Test
    fun test_select_wireguard_port() = runTest {
        // Arrange
        val wireguardPort: Constraint<Port> = Constraint.Only(Port(99))
        val wireguardConstraints = WireguardConstraints(port = wireguardPort)
        every { mockRelayListUseCase.updateSelectedWireguardConstraints(any()) } returns Unit

        // Act
        viewModel.onWireguardPortSelected(wireguardPort)

        // Assert
        verify(exactly = 1) {
            mockRelayListUseCase.updateSelectedWireguardConstraints(wireguardConstraints)
        }
    }

    @Test
    fun `given useCase systemVpnSettingsAvailable is true then uiState should be systemVpnSettingsAvailable=true`() =
        runTest {
            val systemVpnSettingsAvailable = true

            every { mockSystemVpnSettingsUseCase.systemVpnSettingsAvailable() } returns
                systemVpnSettingsAvailable

            viewModel.uiState.test {
                assertEquals(systemVpnSettingsAvailable, awaitItem().systemVpnSettingsAvailable)
            }
        }
}
