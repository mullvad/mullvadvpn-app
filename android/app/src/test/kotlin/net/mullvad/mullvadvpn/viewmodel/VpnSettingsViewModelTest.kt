package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import arrow.core.right
import io.mockk.Awaits
import io.mockk.Runs
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.just
import io.mockk.mockk
import io.mockk.unmockkAll
import io.mockk.verify
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DaitaSettings
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.lib.model.RelayConstraints
import net.mullvad.mullvadvpn.lib.model.RelaySettings
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.model.TunnelOptions
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints
import net.mullvad.mullvadvpn.lib.model.WireguardTunnelOptions
import net.mullvad.mullvadvpn.repository.AutoStartAndConnectOnBootRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.usecase.SystemVpnSettingsAvailableUseCase
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExperimentalCoroutinesApi
@ExtendWith(TestCoroutineRule::class)
class VpnSettingsViewModelTest {

    private val mockSettingsRepository: SettingsRepository = mockk()
    private val mockSystemVpnSettingsUseCase: SystemVpnSettingsAvailableUseCase =
        mockk(relaxed = true)
    private val mockRelayListRepository: RelayListRepository = mockk()
    private val mockAutoStartAndConnectOnBootRepository: AutoStartAndConnectOnBootRepository =
        mockk()
    private val mockWireguardConstraintsRepository: WireguardConstraintsRepository = mockk()

    private val mockSettingsUpdate = MutableStateFlow<Settings?>(null)
    private val portRangeFlow = MutableStateFlow(emptyList<PortRange>())
    private val autoStartAndConnectOnBootFlow = MutableStateFlow(false)

    private lateinit var viewModel: VpnSettingsViewModel

    @BeforeEach
    fun setup() {
        every { mockSettingsRepository.settingsUpdates } returns mockSettingsUpdate
        every { mockRelayListRepository.portRanges } returns portRangeFlow
        every { mockAutoStartAndConnectOnBootRepository.autoStartAndConnectOnBoot } returns
            autoStartAndConnectOnBootFlow

        viewModel =
            VpnSettingsViewModel(
                repository = mockSettingsRepository,
                systemVpnSettingsUseCase = mockSystemVpnSettingsUseCase,
                relayListRepository = mockRelayListRepository,
                dispatcher = UnconfinedTestDispatcher(),
                autoStartAndConnectOnBootRepository = mockAutoStartAndConnectOnBootRepository,
                wireguardConstraintsRepository = mockWireguardConstraintsRepository,
            )
    }

    @AfterEach
    fun tearDown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `onSelectCustomTcpOverUdpPort should invoke setCustomObfuscationPort on SettingsRepository`() =
        runTest {
            val customPort = Port(5001)
            coEvery {
                mockSettingsRepository.setCustomUdp2TcpObfuscationPort(Constraint.Only(customPort))
            } returns Unit.right()
            viewModel.onObfuscationPortSelected(Constraint.Only(customPort))
            coVerify(exactly = 1) {
                mockSettingsRepository.setCustomUdp2TcpObfuscationPort(Constraint.Only(customPort))
            }
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
            // Can not use a mock here since mocking a value class val leads to class cast exception
            val mockWireguardTunnelOptions =
                WireguardTunnelOptions(
                    mtu = Mtu(0),
                    quantumResistant = expectedResistantState,
                    daitaSettings = DaitaSettings(enabled = false, directOnly = false),
                )

            every { mockSettings.tunnelOptions } returns mockTunnelOptions
            every { mockTunnelOptions.wireguard } returns mockWireguardTunnelOptions
            every { mockSettings.relaySettings } returns mockk<RelaySettings>(relaxed = true)
            every { mockSettings.relaySettings.relayConstraints.wireguardConstraints.port } returns
                Constraint.Any

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
            val expectedPort = Constraint.Only(Port(99))
            val mockSettings: Settings = mockk(relaxed = true)
            val mockRelaySettings: RelaySettings = mockk()
            val mockRelayConstraints: RelayConstraints = mockk()
            val mockWireguardConstraints: WireguardConstraints = mockk()

            every { mockSettings.relaySettings } returns mockRelaySettings
            every { mockRelaySettings.relayConstraints } returns mockRelayConstraints
            every { mockRelayConstraints.wireguardConstraints } returns mockWireguardConstraints
            every { mockWireguardConstraints.port } returns expectedPort
            every { mockWireguardConstraints.ipVersion } returns Constraint.Any
            every { mockSettings.tunnelOptions } returns
                TunnelOptions(
                    wireguard =
                        WireguardTunnelOptions(
                            mtu = null,
                            quantumResistant = QuantumResistantState.Off,
                            daitaSettings = DaitaSettings(enabled = false, directOnly = false),
                        ),
                    dnsOptions = mockk(relaxed = true),
                    genericOptions = mockk(relaxed = true),
                )

            // Act, Assert
            viewModel.uiState.test {
                assertIs<Constraint.Any>(awaitItem().selectedWireguardPort)
                mockSettingsUpdate.value = mockSettings
                assertEquals(expectedPort.value, awaitItem().customWireguardPort)
                assertEquals(expectedPort, awaitItem().selectedWireguardPort)
            }
        }

    @Test
    fun `onWireguardPortSelected should invoke setWireguardPort with Constraint Only with same port`() =
        runTest {
            // Arrange
            val wireguardPort: Constraint<Port> = Constraint.Only(Port(99))
            val wireguardConstraints =
                WireguardConstraints(
                    port = wireguardPort,
                    isMultihopEnabled = false,
                    entryLocation = Constraint.Any,
                    ipVersion = Constraint.Any,
                )
            coEvery { mockWireguardConstraintsRepository.setWireguardPort(any()) } returns
                Unit.right()

            // Act
            viewModel.onWireguardPortSelected(wireguardPort)

            // Assert
            coVerify(exactly = 1) {
                mockWireguardConstraintsRepository.setWireguardPort(wireguardConstraints.port)
            }
        }

    @Test
    fun `when useCase systemVpnSettingsAvailable is true then uiState should be systemVpnSettingsAvailable=true`() =
        runTest {
            val systemVpnSettingsAvailable = true

            every { mockSystemVpnSettingsUseCase() } returns systemVpnSettingsAvailable

            viewModel.uiState.test {
                assertEquals(systemVpnSettingsAvailable, awaitItem().systemVpnSettingsAvailable)
            }
        }

    @Test
    fun `when autoStartAndConnectOnBoot is true then uiState should be autoStart=true`() = runTest {
        // Arrange
        val connectOnStart = true

        // Act
        autoStartAndConnectOnBootFlow.value = connectOnStart

        // Assert
        viewModel.uiState.test {
            assertEquals(connectOnStart, awaitItem().autoStartAndConnectOnBoot)
        }
    }

    @Test
    fun `calling onToggleAutoStartAndConnectOnBoot should call autoStartAndConnectOnBoot`() =
        runTest {
            // Arrange
            val targetState = true
            every {
                mockAutoStartAndConnectOnBootRepository.setAutoStartAndConnectOnBoot(targetState)
            } just Runs

            // Act
            viewModel.onToggleAutoStartAndConnectOnBoot(targetState)

            // Assert
            verify {
                mockAutoStartAndConnectOnBootRepository.setAutoStartAndConnectOnBoot(targetState)
            }
        }

    @Test
    fun `when device ip version is IPv6 then UiState should be IPv6`() = runTest {
        // Arrange
        val ipVersion = Constraint.Only(IpVersion.IPV6)
        val mockSettings = mockk<Settings>(relaxed = true)
        every { mockSettings.relaySettings.relayConstraints.wireguardConstraints.ipVersion } returns
            ipVersion
        every { mockSettings.tunnelOptions.wireguard } returns
            WireguardTunnelOptions(
                mtu = Mtu(0),
                quantumResistant = QuantumResistantState.Off,
                daitaSettings = DaitaSettings(enabled = false, directOnly = false),
            )
        every { mockSettings.relaySettings.relayConstraints.wireguardConstraints.port } returns
            Constraint.Any

        // Act, Assert
        viewModel.uiState.test {
            // Default value
            awaitItem()
            mockSettingsUpdate.value = mockSettings
            assertEquals(ipVersion, awaitItem().deviceIpVersion)
        }
    }

    @Test
    fun `calling onDeviceIpVersionSelected should call setDeviceIpVersion`() = runTest {
        // Arrange
        val targetState = Constraint.Only(IpVersion.IPV4)
        coEvery { mockWireguardConstraintsRepository.setDeviceIpVersion(targetState) } just Awaits

        // Act
        viewModel.onDeviceIpVersionSelected(targetState)

        // Assert
        coVerify(exactly = 1) { mockWireguardConstraintsRepository.setDeviceIpVersion(targetState) }
    }
}
