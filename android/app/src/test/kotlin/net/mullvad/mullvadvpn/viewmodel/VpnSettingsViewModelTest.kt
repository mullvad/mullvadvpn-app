package net.mullvad.mullvadvpn.viewmodel

import android.content.res.Resources
import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import io.mockk.verify
import kotlin.test.assertEquals
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.TestCoroutineRule
import net.mullvad.mullvadvpn.model.QuantumResistantState
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.model.TunnelOptions
import net.mullvad.mullvadvpn.model.WireguardTunnelOptions
import net.mullvad.mullvadvpn.repository.SettingsRepository
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

    private val mockSettingsUpdate = MutableStateFlow<Settings?>(null)

    private lateinit var viewModel: VpnSettingsViewModel

    @Before
    fun setUp() {
        every { mockSettingsRepository.settingsUpdates } returns mockSettingsUpdate

        viewModel =
            VpnSettingsViewModel(
                repository = mockSettingsRepository,
                inetAddressValidator = mockInetAddressValidator,
                resources = mockResources,
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
        every {
            mockSettingsRepository.setWireguardQuantumResistant(quantumResistantState)
        } returns Unit
        viewModel.onSelectQuantumResistanceSetting(quantumResistantState)
        verify(exactly = 1) {
            mockSettingsRepository.setWireguardQuantumResistant(quantumResistantState)
        }
    }

    @Test
    fun test_update_quantum_resistant_default_state() = runTest {
        val expectedResistantState = QuantumResistantState.Off
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

        viewModel.uiState.test {
            assertEquals(defaultResistantState, awaitItem().quantumResistant)
            mockSettingsUpdate.value = mockSettings
            assertEquals(expectedResistantState, awaitItem().quantumResistant)
        }
    }
}
