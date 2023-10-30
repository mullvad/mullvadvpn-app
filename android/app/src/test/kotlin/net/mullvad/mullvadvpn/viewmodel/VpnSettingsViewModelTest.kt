package net.mullvad.mullvadvpn.viewmodel

import android.content.res.Resources
import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import io.mockk.slot
import io.mockk.unmockkAll
import io.mockk.verify
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.VpnSettingsDialog
import net.mullvad.mullvadvpn.compose.state.VpnSettingsUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.assertLists
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
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import org.apache.commons.validator.routines.InetAddressValidator
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class VpnSettingsViewModelTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    private val mockSettingsRepository: SettingsRepository = mockk()
    private val mockInetAddressValidator: InetAddressValidator = mockk()
    private val mockResources: Resources = mockk()
    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()

    private val mockServiceConnectionContainer: ServiceConnectionContainer = mockk()
    private val mockRelayListListener: RelayListListener = mockk()
    private val portRangeSlot = slot<(List<PortRange>) -> Unit>()

    private val mockSettingsUpdate = MutableStateFlow<Settings?>(null)
    private val mockConnectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Disconnected)

    private lateinit var viewModel: VpnSettingsViewModel

    @Before
    fun setUp() {
        every { mockSettingsRepository.settingsUpdates } returns mockSettingsUpdate
        every { mockServiceConnectionManager.connectionState } returns mockConnectionState

        every { mockServiceConnectionContainer.relayListListener } returns mockRelayListListener

        every { mockRelayListListener.onPortRangesChange = capture(portRangeSlot) } answers {}
        every { mockRelayListListener.onPortRangesChange = null } answers {}

        viewModel =
            VpnSettingsViewModel(
                repository = mockSettingsRepository,
                inetAddressValidator = mockInetAddressValidator,
                resources = mockResources,
                serviceConnectionManager = mockServiceConnectionManager,
                dispatcher = UnconfinedTestDispatcher()
            )
    }

    @After
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
            mockConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            portRangeSlot.captured.invoke(emptyList())
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
            mockConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            portRangeSlot.captured.invoke(emptyList())
            assertEquals(expectedPort, awaitItem().selectedWireguardPort)
        }
    }

    @Test
    fun test_select_wireguard_port() = runTest {
        // Arrange
        val wireguardPort: Constraint<Port> = Constraint.Only(Port(99))
        val wireguardConstraints = WireguardConstraints(port = wireguardPort)
        every {
            mockRelayListListener.updateSelectedWireguardConstraints(wireguardConstraints)
        } returns Unit

        // Act
        mockConnectionState.value =
            ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
        viewModel.onWireguardPortSelected(wireguardPort)

        // Assert
        verify(exactly = 1) {
            mockRelayListListener.updateSelectedWireguardConstraints(wireguardConstraints)
        }
    }

    @Test
    fun test_update_port_range_state() = runTest {
        // Arrange
        val expectedPortRange = listOf<PortRange>(mockk(), mockk())
        val mockSettings: Settings = mockk(relaxed = true)

        every { mockSettings.relaySettings } returns mockk<RelaySettings.Normal>(relaxed = true)

        // Act, Assert
        viewModel.uiState.test {
            assertIs<VpnSettingsUiState>(awaitItem())
            mockSettingsUpdate.value = mockSettings
            viewModel.onWireguardPortInfoClicked()
            mockConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            portRangeSlot.captured.invoke(expectedPortRange)
            val state = awaitItem()
            assertTrue { state.dialog is VpnSettingsDialog.WireguardPortInfo }
            assertLists(expectedPortRange, state.availablePortRanges)
        }
    }
}
