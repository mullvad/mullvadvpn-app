package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import arrow.core.right
import com.ramcosta.composedestinations.generated.navargs.toSavedStateHandle
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.screen.DaitaNavArgs
import net.mullvad.mullvadvpn.compose.state.DaitaUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.SettingsRepository
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class DaitaViewModelTest {
    private val mockSettingsRepository: SettingsRepository = mockk()
    private val settings = MutableStateFlow<Settings>(mockk(relaxed = true))

    private lateinit var viewModel: DaitaViewModel

    @BeforeEach
    fun setUp() {
        every { mockSettingsRepository.settingsUpdates } returns settings
        viewModel =
            DaitaViewModel(
                mockSettingsRepository,
                savedStateHandle = DaitaNavArgs().toSavedStateHandle(),
            )
    }

    @Test
    fun `given daita enabled ui state should be daita enabled`() = runTest {
        // Arrange
        val expectedState = DaitaUiState(daitaEnabled = true, directOnly = false)
        settings.value = mockk {
            every { tunnelOptions.wireguard.daitaSettings } returns
                mockk {
                    every { enabled } returns true
                    every { directOnly } returns false
                }
        }

        // Act, Assert
        viewModel.uiState.test { assertEquals(expectedState, awaitItem()) }
    }

    @Test
    fun `given direct only enabled ui state should be direct only enabled`() = runTest {
        // Arrange
        val expectedState = DaitaUiState(daitaEnabled = false, directOnly = true)
        settings.value = mockk {
            every { tunnelOptions.wireguard.daitaSettings } returns
                mockk {
                    every { enabled } returns false
                    every { directOnly } returns true
                }
        }

        // Act, Assert
        viewModel.uiState.test { assertEquals(expectedState, awaitItem()) }
    }

    @Test
    fun `set daita should call settings repository set daita enabled`() {
        // Arrange
        coEvery { mockSettingsRepository.setDaitaEnabled(any()) } returns Unit.right()

        // Act
        viewModel.setDaita(true)

        // Assert
        coVerify { mockSettingsRepository.setDaitaEnabled(true) }
    }

    @Test
    fun `set direct only should call settings repository set daita direct only`() {
        // Arrange
        coEvery { mockSettingsRepository.setDaitaDirectOnly(any()) } returns Unit.right()

        // Act
        viewModel.setDirectOnly(true)

        // Assert
        coVerify { mockSettingsRepository.setDaitaDirectOnly(true) }
    }
}
