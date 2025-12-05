package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import com.ramcosta.composedestinations.generated.destinations.ConnectDestination
import com.ramcosta.composedestinations.generated.navargs.toSavedStateHandle
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.screen.AntiCensorshipSettingsNavArgs
import net.mullvad.mullvadvpn.compose.state.AntiCensorshipSettingsUiState
import net.mullvad.mullvadvpn.compose.state.ObfuscationSettingItem
import net.mullvad.mullvadvpn.compose.util.BackstackObserver
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DaitaSettings
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.model.TunnelOptions
import net.mullvad.mullvadvpn.repository.AutoStartAndConnectOnBootRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.util.Lc
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertInstanceOf
import org.junit.jupiter.api.extension.ExtendWith

@ExperimentalCoroutinesApi
@ExtendWith(TestCoroutineRule::class)
class AntiCensorshipSettingsViewModelTest {

    private val mockSettingsRepository: SettingsRepository = mockk()
    private val mockAutoStartAndConnectOnBootRepository: AutoStartAndConnectOnBootRepository =
        mockk()
    private val mockBackstackObserver: BackstackObserver = mockk()

    private val mockSettingsUpdate = MutableStateFlow<Settings?>(null)
    private val autoStartAndConnectOnBootFlow = MutableStateFlow(false)
    private val previousDestinationFlow = MutableStateFlow(ConnectDestination)

    private lateinit var viewModel: AntiCensorshipSettingsViewModel

    @BeforeEach
    fun setup() {
        every { mockSettingsRepository.settingsUpdates } returns mockSettingsUpdate
        every { mockAutoStartAndConnectOnBootRepository.autoStartAndConnectOnBoot } returns
            autoStartAndConnectOnBootFlow
        every { mockBackstackObserver.previousDestinationFlow } returns previousDestinationFlow

        viewModel =
            AntiCensorshipSettingsViewModel(
                settingsRepository = mockSettingsRepository,
                savedStateHandle = AntiCensorshipSettingsNavArgs().toSavedStateHandle(),
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
    fun `when SettingsRepository emits WireguardPort as obfuscation mode then uiState should return WireguardPort as selected and with the correct port`() =
        runTest {
            // Arrange
            val expectedPort = Constraint.Only(Port(99))
            val mockSettings: Settings = mockk(relaxed = true)

            every { mockSettings.obfuscationSettings.wireguardPort } returns expectedPort
            every { mockSettings.obfuscationSettings.selectedObfuscationMode } returns
                ObfuscationMode.WireguardPort
            every { mockSettings.tunnelOptions } returns
                TunnelOptions(
                    mtu = null,
                    quantumResistant = QuantumResistantState.Off,
                    daitaSettings = DaitaSettings(enabled = false, directOnly = false),
                    dnsOptions = mockk(relaxed = true),
                    enableIpv6 = true,
                )

            // Act, Assert
            viewModel.uiState.test {
                assertInstanceOf<Lc.Loading<Boolean>>(awaitItem())

                mockSettingsUpdate.value = mockSettings

                with(awaitItem()) {
                    assertInstanceOf<Lc.Content<AntiCensorshipSettingsUiState>>(this)
                    val customPortSetting =
                        value.items
                            .filterIsInstance<ObfuscationSettingItem.Obfuscation.WireguardPort>()
                            .first()

                    // Port should be what we expect and be selected
                    assertEquals(
                        expectedPort.value.value,
                        customPortSetting.port.getOrNull()!!.value,
                    )
                    assertTrue(customPortSetting.selected)
                }
            }
        }
}
