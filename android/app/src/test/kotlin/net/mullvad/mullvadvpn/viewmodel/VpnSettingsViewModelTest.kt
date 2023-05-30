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
import net.mullvad.mullvadvpn.model.CustomDnsOptions
import net.mullvad.mullvadvpn.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.DnsState
import net.mullvad.mullvadvpn.model.ObfuscationSettings
import net.mullvad.mullvadvpn.model.QuantumResistantState
import net.mullvad.mullvadvpn.model.SelectedObfuscation
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

    // Flows
    private val mockSettingsUpdate = MutableStateFlow<Settings?>(null)

    private lateinit var testSubject: VpnSettingsViewModel

    @Before
    fun setUp() {
        every { mockSettingsRepository.settingsUpdates } returns mockSettingsUpdate

        testSubject =
            VpnSettingsViewModel(
                repository = mockSettingsRepository,
                inetAddressValidator = mockInetAddressValidator,
                resources = mockResources,
                dispatcher = UnconfinedTestDispatcher()
            )
    }

    @After
    fun tearDown() {
        testSubject.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun test_select_quantum_resistant_state_select() = runTest {
        val quantumResistantState = QuantumResistantState.On
        testSubject.onSelectQuantumResistanceSetting(quantumResistantState)
        verify(exactly = 1) {
            mockSettingsRepository.setWireguardQuantumResistant(quantumResistantState)
        }
    }

    @Test
    fun test_update_quantum_resistant_default_state() = runTest {
        val expectedResistantState = QuantumResistantState.Off
        testSubject.uiState.test {
            assertEquals(expectedResistantState, awaitItem().quantumResistant)
        }
    }

    @Test
    fun test_update_quantum_resistant_update_state() = runTest {
        val defaultResistantState = QuantumResistantState.Off
        val expectedResistantState = QuantumResistantState.On
        val mockSettings: Settings = mockk()
        val mockTunnelOptions: TunnelOptions = mockk()
        val mockDnsOptions: DnsOptions = mockk()
        val mockCustomDnsOptions: CustomDnsOptions = mockk()
        val mockWireguardTunnelOptions: WireguardTunnelOptions = mockk()
        val mockDefaultDnsOptions: DefaultDnsOptions = mockk()
        val mockObfuscationSettings: ObfuscationSettings = mockk()
        every { mockSettings.tunnelOptions } returns mockTunnelOptions
        every { mockSettings.autoConnect } returns false
        every { mockSettings.allowLan } returns false
        every { mockSettings.obfuscationSettings } returns mockObfuscationSettings
        every { mockObfuscationSettings.selectedObfuscation } returns SelectedObfuscation.Auto
        every { mockTunnelOptions.dnsOptions } returns mockDnsOptions
        every { mockDnsOptions.state } returns DnsState.Default
        every { mockDnsOptions.customOptions } returns mockCustomDnsOptions
        every { mockCustomDnsOptions.addresses } returns ArrayList()
        every { mockDnsOptions.defaultOptions } returns mockDefaultDnsOptions
        every { mockTunnelOptions.wireguard } returns mockWireguardTunnelOptions
        every { mockWireguardTunnelOptions.mtu } returns 100
        every { mockWireguardTunnelOptions.quantumResistant } returns expectedResistantState
        testSubject.uiState.test {
            assertEquals(defaultResistantState, awaitItem().quantumResistant)
            mockSettingsUpdate.value = mockSettings
            assertEquals(expectedResistantState, awaitItem().quantumResistant)
        }
    }
}
