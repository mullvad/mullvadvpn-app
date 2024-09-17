package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.SettingsRepository
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class Udp2TcpSettingsViewModelTest {

    private val mockSettingsRepository: SettingsRepository = mockk()

    private val settingsFlow = MutableStateFlow<Settings?>(null)

    private lateinit var viewModel: Udp2TcpSettingsViewModel

    @BeforeEach
    fun setUp() {
        every { mockSettingsRepository.settingsUpdates } returns settingsFlow

        viewModel = Udp2TcpSettingsViewModel(repository = mockSettingsRepository)
    }

    @Test
    fun `uiState should reflect latest value from settings`() = runTest {
        // Arrange
        val mockSettings: Settings = mockk()
        val port = Port(123)
        every { mockSettings.obfuscationSettings.udp2tcp.port } returns Constraint.Only(port)

        settingsFlow.update { mockSettings }

        // Act, Assert
        viewModel.uiState.test {
            // Check result
            val result = awaitItem().port
            assertEquals(Constraint.Only(port), result)
        }
    }

    @Test
    fun `when onObfuscationPortSelected is called should call repository`() {
        // Arrange
        val port = Constraint.Only(Port(123))
        coEvery { mockSettingsRepository.setCustomUdp2TcpObfuscationPort(port) } returns
            Unit.right()

        // Act
        viewModel.onObfuscationPortSelected(port)

        // Assert
        coVerify { mockSettingsRepository.setCustomUdp2TcpObfuscationPort(port) }
    }
}
