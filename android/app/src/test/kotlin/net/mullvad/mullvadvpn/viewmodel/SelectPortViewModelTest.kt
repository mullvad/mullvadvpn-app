package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import arrow.core.right
import com.ramcosta.composedestinations.generated.destinations.ConnectDestination
import com.ramcosta.composedestinations.generated.navargs.toSavedStateHandle
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.test.assertIs
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.screen.SelectPortNavArgs
import net.mullvad.mullvadvpn.compose.state.SelectPortUiState
import net.mullvad.mullvadvpn.compose.util.BackstackObserver
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.PortType
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.AutoStartAndConnectOnBootRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.util.Lc
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertInstanceOf
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class SelectPortViewModelTest {

    private val mockSettingsRepository: SettingsRepository = mockk()
    private val mockRelayListRepository: RelayListRepository = mockk()
    private val mockAutoStartAndConnectOnBootRepository: AutoStartAndConnectOnBootRepository =
        mockk()
    private val mockBackstackObserver: BackstackObserver = mockk()

    private val settingsFlow = MutableStateFlow<Settings?>(null)
    private val autoStartAndConnectOnBootFlow = MutableStateFlow(false)
    private val previousDestinationFlow = MutableStateFlow(ConnectDestination)

    private val portRangeFlow = MutableStateFlow(emptyList<PortRange>())
    private val shadowsocksPortRangeFlow = MutableStateFlow(emptyList<PortRange>())

    private lateinit var viewModel: SelectPortViewModel

    private fun setViewModel(navArgs: SelectPortNavArgs) {
        viewModel =
            SelectPortViewModel(
                settingsRepository = mockSettingsRepository,
                resources = mockk(relaxed = true),
                relayListRepository = mockRelayListRepository,
                savedStateHandle = navArgs.toSavedStateHandle(),
            )
    }

    @BeforeEach
    fun setup() {
        every { mockSettingsRepository.settingsUpdates } returns settingsFlow
        every { mockRelayListRepository.portRanges } returns portRangeFlow
        every { mockRelayListRepository.shadowsocksPortRanges } returns shadowsocksPortRangeFlow
        every { mockAutoStartAndConnectOnBootRepository.autoStartAndConnectOnBoot } returns
            autoStartAndConnectOnBootFlow
        every { mockBackstackObserver.previousDestinationFlow } returns previousDestinationFlow
    }

    @AfterEach
    fun tearDown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `initial state should be loading`() = runTest {
        setViewModel(SelectPortNavArgs(portType = PortType.Wireguard))
        viewModel.uiState.test { assertInstanceOf<Lc.Loading<Boolean>>(awaitItem()) }
    }

    @Test
    fun `uiState should reflect latest port value from settings`() = runTest {
        setViewModel(SelectPortNavArgs(portType = PortType.Shadowsocks))

        // Arrange
        val mockSettings: Settings = mockk()
        val port = Port(123)
        every { mockSettings.obfuscationSettings.shadowsocks.port } returns Constraint.Only(port)

        settingsFlow.update { mockSettings }

        // Act, Assert
        viewModel.uiState.test {
            val result = awaitItem()
            assertIs<Lc.Content<SelectPortUiState>>(result)
            assertEquals(Constraint.Only(port), result.value.port)
        }
    }

    @Test
    fun `selecting custom Udp2Tcp port should invoke setCustomUdp2TcpObfuscationPort on SettingsRepository`() =
        runTest {
            setViewModel(SelectPortNavArgs(portType = PortType.Udp2Tcp))
            val customPort = Port(5001)
            coEvery {
                mockSettingsRepository.setCustomUdp2TcpObfuscationPort(Constraint.Only(customPort))
            } returns Unit.right()
            viewModel.onPortSelected(Constraint.Only(customPort))
            coVerify(exactly = 1) {
                mockSettingsRepository.setCustomUdp2TcpObfuscationPort(Constraint.Only(customPort))
            }
        }

    @Test
    fun `setting custom Wireguard port should invoke setWireguardPort on SettingsRepository`() =
        runTest {
            // Arrange
            setViewModel(SelectPortNavArgs(portType = PortType.Wireguard))
            val wireguardPort: Constraint<Port> = Constraint.Only(Port(99))
            coEvery { mockSettingsRepository.setCustomWireguardPort(any()) } returns Unit.right()

            // Act
            viewModel.onPortSelected(wireguardPort)

            // Assert
            coVerify(exactly = 1) { mockSettingsRepository.setCustomWireguardPort(wireguardPort) }
        }

    @Test
    fun `setting custom Shadowsocks port should invoke setWireguardPort on SettingsRepository`() =
        runTest {
            // Arrange
            setViewModel(SelectPortNavArgs(portType = PortType.Shadowsocks))
            val customPort: Constraint<Port> = Constraint.Only(Port(51900))
            coEvery { mockSettingsRepository.setCustomShadowsocksObfuscationPort(any()) } returns
                Unit.right()

            // Act
            viewModel.onPortSelected(customPort)

            // Assert
            coVerify(exactly = 1) {
                mockSettingsRepository.setCustomShadowsocksObfuscationPort(customPort)
            }
        }

    @Test
    fun `when reset custom port is called should reset custom port`() = runTest {
        // Arrange
        setViewModel(SelectPortNavArgs(portType = PortType.Shadowsocks))
        val mockSettings: Settings = mockk()
        val port = Port(123)
        every { mockSettings.obfuscationSettings.shadowsocks.port } returns Constraint.Only(port)
        coEvery {
            mockSettingsRepository.setCustomShadowsocksObfuscationPort(Constraint.Any)
        } returns Unit.right()

        settingsFlow.update { mockSettings }

        // Act, Assert
        viewModel.uiState.test {
            val startState = awaitItem()
            assertIs<Lc.Content<SelectPortUiState>>(startState)
            assertEquals(port, startState.value.customPort)

            viewModel.resetCustomPort()

            val updatedState = awaitItem()
            assertIs<Lc.Content<SelectPortUiState>>(updatedState)
            assertEquals(null, updatedState.value.customPort)
            coVerify { mockSettingsRepository.setCustomShadowsocksObfuscationPort(Constraint.Any) }
        }
    }
}
