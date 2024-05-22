package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.Mtu
import net.mullvad.mullvadvpn.model.Port
import net.mullvad.mullvadvpn.model.PortRange
import net.mullvad.mullvadvpn.model.QuantumResistantState
import net.mullvad.mullvadvpn.model.RelayConstraints
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.model.TunnelOptions
import net.mullvad.mullvadvpn.model.WireguardConstraints
import net.mullvad.mullvadvpn.model.WireguardTunnelOptions
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.usecase.PortRangeUseCase
import net.mullvad.mullvadvpn.usecase.SystemVpnSettingsUseCase
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class VpnSettingsViewModelTest {

    private val mockSettingsRepository: SettingsRepository = mockk()
    private val mockPortRangeUseCase: PortRangeUseCase = mockk()
    private val mockSystemVpnSettingsUseCase: SystemVpnSettingsUseCase = mockk(relaxed = true)
    private val mockRelayListRepository: RelayListRepository = mockk()

    private val mockSettingsUpdate = MutableStateFlow<Settings?>(null)
    private val portRangeFlow = MutableStateFlow(emptyList<PortRange>())

    private lateinit var viewModel: VpnSettingsViewModel

    @BeforeEach
    fun setup() {
        every { mockSettingsRepository.settingsUpdates } returns mockSettingsUpdate
        every { mockPortRangeUseCase.portRanges() } returns portRangeFlow

        viewModel =
            VpnSettingsViewModel(
                repository = mockSettingsRepository,
                portRangeUseCase = mockPortRangeUseCase,
                systemVpnSettingsUseCase = mockSystemVpnSettingsUseCase,
                relayListRepository = mockRelayListRepository,
                dispatcher = UnconfinedTestDispatcher()
            )
    }

    @AfterEach
    fun tearDown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `onSelectQuantumResistanceSetting should invoke setWireguardQuantumResistant on SettingsRepository`() =
        runTest {
            val quantumResistantState = QuantumResistantState.On
            coEvery {
                mockSettingsRepository.setWireguardQuantumResistant(quantumResistantState)
            } returns Unit.right()
            viewModel.onSelectQuantumResistanceSetting(quantumResistantState)
            coVerify(exactly = 1) {
                mockSettingsRepository.setWireguardQuantumResistant(quantumResistantState)
            }
        }

    @Test
    fun `quantumResistant should be Off in uiState in initial state`() = runTest {
        // Arrange
        val expectedResistantState = QuantumResistantState.Off

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(expectedResistantState, awaitItem().quantumResistant)
        }
    }

    @Test
    fun `when SettingsRepository emits quantumResistant On uiState should emit quantumResistant On`() =
        runTest {
            val defaultResistantState = QuantumResistantState.Off
            val expectedResistantState = QuantumResistantState.On
            val mockSettings: Settings = mockk(relaxed = true)
            val mockTunnelOptions: TunnelOptions = mockk(relaxed = true)
            val mockWireguardTunnelOptions: WireguardTunnelOptions = mockk(relaxed = true)

            every { mockSettings.tunnelOptions } returns mockTunnelOptions
            every { mockTunnelOptions.wireguard } returns mockWireguardTunnelOptions
            every { mockWireguardTunnelOptions.quantumResistant } returns expectedResistantState
            every { mockWireguardTunnelOptions.mtu } returns Mtu(0)
            every { mockSettings.relaySettings } returns mockk<RelaySettings>(relaxed = true)


            viewModel.uiState.test {
                assertEquals(defaultResistantState, awaitItem().quantumResistant)
                mockSettingsUpdate.value = mockSettings
                assertEquals(expectedResistantState, awaitItem().quantumResistant)
            }
        }

    @Test
    fun `when SettingsRepository emits Constraint Only then uiState should emit custom and selectedWireguardPort with port of Constraint`() =
        runTest {
            // Arrange
            val expectedPort: Constraint<Port> = Constraint.Only(Port(99))
            val mockSettings: Settings = mockk(relaxed = true)
            val mockRelaySettings: RelaySettings = mockk()
            val mockRelayConstraints: RelayConstraints = mockk()
            val mockWireguardConstraints: WireguardConstraints = mockk()

            every { mockSettings.relaySettings } returns mockRelaySettings
            every { mockRelaySettings.relayConstraints } returns mockRelayConstraints
            every { mockRelayConstraints.wireguardConstraints } returns mockWireguardConstraints
            every { mockWireguardConstraints.port } returns expectedPort
            every { mockSettings.tunnelOptions } returns
                TunnelOptions(
                    wireguard =
                        WireguardTunnelOptions(
                            mtu = null,
                            quantumResistant = QuantumResistantState.Off
                        ),
                    dnsOptions = mockk(relaxed = true)
                )

            // Act, Assert
            viewModel.uiState.test {
                assertIs<Constraint.Any>(awaitItem().selectedWireguardPort)
                mockSettingsUpdate.value = mockSettings
                assertEquals(expectedPort, awaitItem().customWireguardPort)
                assertEquals(expectedPort, awaitItem().selectedWireguardPort)
            }
        }

    @Test
    fun `onWireguardPortSelected should invoke updateSelectedWireguardConstraint with Constraint Only with same port`() =
        runTest {
            // Arrange
            val wireguardPort: Constraint<Port> = Constraint.Only(Port(99))
            val wireguardConstraints = WireguardConstraints(port = wireguardPort)
            coEvery { mockRelayListRepository.updateSelectedWireguardConstraints(any()) } returns
                Unit.right()

            // Act
            viewModel.onWireguardPortSelected(wireguardPort)

            // Assert
            coVerify(exactly = 1) {
                mockRelayListRepository.updateSelectedWireguardConstraints(wireguardConstraints)
            }
        }

    @Test
    fun `when useCase systemVpnSettingsAvailable is true then uiState should be systemVpnSettingsAvailable=true`() =
        runTest {
            val systemVpnSettingsAvailable = true

            every { mockSystemVpnSettingsUseCase.systemVpnSettingsAvailable() } returns
                systemVpnSettingsAvailable

            viewModel.uiState.test {
                assertEquals(systemVpnSettingsAvailable, awaitItem().systemVpnSettingsAvailable)
            }
        }
}
