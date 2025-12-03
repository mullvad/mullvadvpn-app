package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import arrow.core.right
import com.ramcosta.composedestinations.generated.destinations.ConnectDestination
import com.ramcosta.composedestinations.generated.navargs.toSavedStateHandle
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
import kotlin.test.assertTrue
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.screen.VpnSettingsNavArgs
import net.mullvad.mullvadvpn.compose.state.VpnSettingItem
import net.mullvad.mullvadvpn.compose.state.VpnSettingsUiState
import net.mullvad.mullvadvpn.compose.util.BackstackObserver
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DaitaSettings
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.ObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.lib.model.Recents
import net.mullvad.mullvadvpn.lib.model.RelayConstraints
import net.mullvad.mullvadvpn.lib.model.RelaySettings
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.model.ShadowsocksObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.SplitTunnelSettings
import net.mullvad.mullvadvpn.lib.model.TunnelOptions
import net.mullvad.mullvadvpn.lib.model.Udp2TcpObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints
import net.mullvad.mullvadvpn.repository.AutoStartAndConnectOnBootRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.usecase.SystemVpnSettingsAvailableUseCase
import net.mullvad.mullvadvpn.util.Lc
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertInstanceOf
import org.junit.jupiter.api.extension.ExtendWith

@ExperimentalCoroutinesApi
@ExtendWith(TestCoroutineRule::class)
class VpnSettingsViewModelTest {

    private val mockSettingsRepository: SettingsRepository = mockk()
    private val mockSystemVpnSettingsUseCase: SystemVpnSettingsAvailableUseCase =
        mockk(relaxed = true)
    private val mockAutoStartAndConnectOnBootRepository: AutoStartAndConnectOnBootRepository =
        mockk()
    private val mockWireguardConstraintsRepository: WireguardConstraintsRepository = mockk()
    private val mockBackstackObserver: BackstackObserver = mockk()

    private val mockSettingsUpdate = MutableStateFlow<Settings?>(null)
    //    private val portRangeFlow = MutableStateFlow(emptyList<PortRange>())
    private val autoStartAndConnectOnBootFlow = MutableStateFlow(false)
    private val previousDestinationFlow = MutableStateFlow(ConnectDestination)

    private lateinit var viewModel: VpnSettingsViewModel

    @BeforeEach
    fun setup() {
        every { mockSettingsRepository.settingsUpdates } returns mockSettingsUpdate
        //        every { mockRelayListRepository.portRanges } returns portRangeFlow
        every { mockAutoStartAndConnectOnBootRepository.autoStartAndConnectOnBoot } returns
            autoStartAndConnectOnBootFlow
        every { mockBackstackObserver.previousDestinationFlow } returns previousDestinationFlow

        viewModel =
            VpnSettingsViewModel(
                settingsRepository = mockSettingsRepository,
                systemVpnSettingsUseCase = mockSystemVpnSettingsUseCase,
                //                relayListRepository = mockRelayListRepository,
                dispatcher = UnconfinedTestDispatcher(),
                autoStartAndConnectOnBootRepository = mockAutoStartAndConnectOnBootRepository,
                wireguardConstraintsRepository = mockWireguardConstraintsRepository,
                savedStateHandle = VpnSettingsNavArgs().toSavedStateHandle(),
                backstackObserver = mockBackstackObserver,
            )
    }

    @AfterEach
    fun tearDown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `initial state should be loading`() = runTest {
        viewModel.uiState.test { assertInstanceOf<Lc.Loading<Boolean>>(awaitItem()) }
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
    fun `when SettingsRepository emits quantumResistant On uiState should emit quantumResistant On`() =
        runTest {
            val expectedResistantState = QuantumResistantState.On
            val mockSettings: Settings = mockk(relaxed = true)

            // Can not use a mock here since mocking a value class val leads to class cast exception
            every { mockSettings.tunnelOptions } returns
                TunnelOptions(
                    mtu = Mtu(0),
                    quantumResistant = expectedResistantState,
                    daitaSettings = DaitaSettings(enabled = false, directOnly = false),
                    dnsOptions = mockk(relaxed = true),
                    enableIpv6 = true,
                )

            every { mockSettings.relaySettings } returns mockk<RelaySettings>(relaxed = true)
            every { mockSettings.obfuscationSettings.wireguardPort } returns Constraint.Any

            viewModel.uiState.test {
                assertInstanceOf<Lc.Loading<Boolean>>(awaitItem())
                mockSettingsUpdate.value = mockSettings
                val content = awaitItem()
                assertInstanceOf<Lc.Content<VpnSettingsUiState>>(content)

                assertTrue(
                    content.value.settings
                        .filterIsInstance<VpnSettingItem.QuantumItem>()
                        .first { it.quantumResistantState == QuantumResistantState.On }
                        .selected
                )
            }
        }

    @Test
    fun `when useCase systemVpnSettingsAvailable is true then uiState should be systemVpnSettingsAvailable=true`() =
        runTest {
            val systemVpnSettingsAvailable = true

            every { mockSystemVpnSettingsUseCase() } returns systemVpnSettingsAvailable

            viewModel.uiState.test {
                assertInstanceOf<Lc.Loading<Boolean>>(awaitItem())
                mockSettingsUpdate.value = dummySettings

                val content = awaitItem()
                assertInstanceOf<Lc.Content<VpnSettingsUiState>>(content)
                assertTrue(
                    content.value.settings.any { it is VpnSettingItem.AutoConnectAndLockdownMode }
                )
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
            assertInstanceOf<Lc.Loading<Boolean>>(awaitItem())

            mockSettingsUpdate.value = dummySettings
            val content = awaitItem()
            assertInstanceOf<Lc.Content<VpnSettingsUiState>>(content)
            assertTrue(
                content.value.settings.any { it is VpnSettingItem.ConnectDeviceOnStartUpSetting }
            )
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
        every { mockSettings.tunnelOptions } returns
            TunnelOptions(
                mtu = null,
                quantumResistant = QuantumResistantState.Off,
                daitaSettings = DaitaSettings(enabled = false, directOnly = false),
                dnsOptions = mockk(relaxed = true),
                enableIpv6 = true,
            )
        every { mockSettings.obfuscationSettings.wireguardPort } returns Constraint.Any

        // Act, Assert
        viewModel.uiState.test {
            // Loading value
            awaitItem()
            mockSettingsUpdate.value = mockSettings
            val content = awaitItem()
            assertInstanceOf<Lc.Content<VpnSettingsUiState>>(content)
            assertEquals(
                ipVersion,
                content.value.settings
                    .filterIsInstance<VpnSettingItem.DeviceIpVersionItem>()
                    .first { it.selected }
                    .constraint,
            )
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

    companion object {
        val dummySettings: Settings =
            Settings(
                relaySettings =
                    RelaySettings(
                        relayConstraints =
                            RelayConstraints(
                                wireguardConstraints =
                                    WireguardConstraints(
                                        isMultihopEnabled = false,
                                        entryLocation = Constraint.Any,
                                        ipVersion = Constraint.Any,
                                    ),
                                providers = Constraint.Any,
                                ownership = Constraint.Any,
                                location = Constraint.Any,
                            )
                    ),
                obfuscationSettings =
                    ObfuscationSettings(
                        selectedObfuscationMode = ObfuscationMode.Auto,
                        udp2tcp = Udp2TcpObfuscationSettings(Constraint.Any),
                        shadowsocks = ShadowsocksObfuscationSettings(Constraint.Any),
                        wireguardPort = Constraint.Any,
                    ),
                customLists = emptyList(),
                allowLan = false,
                tunnelOptions =
                    TunnelOptions(
                        mtu = null,
                        quantumResistant = QuantumResistantState.On,
                        daitaSettings = DaitaSettings(enabled = false, directOnly = false),
                        dnsOptions = mockk(relaxed = true),
                        enableIpv6 = true,
                    ),
                relayOverrides = emptyList(),
                showBetaReleases = false,
                splitTunnelSettings =
                    SplitTunnelSettings(enabled = false, excludedApps = emptySet()),
                apiAccessMethodSettings = emptyList(),
                recents = Recents.Disabled,
            )
    }
}
