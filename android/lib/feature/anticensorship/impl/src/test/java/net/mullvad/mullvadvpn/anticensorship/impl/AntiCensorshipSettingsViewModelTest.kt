package net.mullvad.mullvadvpn.anticensorship.impl

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import com.ramcosta.composedestinations.generated.anticensorship.navargs.toSavedStateHandle
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DaitaSettings
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.model.TunnelOptions
import net.mullvad.mullvadvpn.lib.repository.AutoStartAndConnectOnBootRepository
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository
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

    private val mockSettingsUpdate = MutableStateFlow<Settings?>(null)
    private val autoStartAndConnectOnBootFlow = MutableStateFlow(false)

    private lateinit var viewModel: AntiCensorshipSettingsViewModel

    @BeforeEach
    fun setup() {
        every { mockSettingsRepository.settingsUpdates } returns mockSettingsUpdate
        every { mockAutoStartAndConnectOnBootRepository.autoStartAndConnectOnBoot } returns
            autoStartAndConnectOnBootFlow

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
