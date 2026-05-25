package net.mullvad.mullvadvpn.feature.lansharing.impl

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertInstanceOf
import org.junit.jupiter.api.extension.ExtendWith

@ExperimentalCoroutinesApi
@ExtendWith(TestCoroutineRule::class)
class LocalNetworkSharingSettingsViewModelTest {

    private val mockSettingsRepository: SettingsRepository = mockk()

    private val mockSettingsUpdate = MutableStateFlow<Settings?>(null)

    private lateinit var viewModel: LocalNetworkSharingViewModel

    @BeforeEach
    fun setup() {
        every { mockSettingsRepository.settingsUpdates } returns mockSettingsUpdate

        viewModel =
            LocalNetworkSharingViewModel(
                isModal = false,
                settingsRepository = mockSettingsRepository,
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
    fun `when lan settings are enabled the ui state should return that they are`() = runTest {
        // Arrange
        val allowLan = true
        val mockSettings: Settings = mockk(relaxed = true)

        every { mockSettings.allowLan } returns allowLan

        // Act, Assert
        viewModel.uiState.test {
            assertInstanceOf<Lc.Loading<Boolean>>(awaitItem())

            mockSettingsUpdate.value = mockSettings

            with(awaitItem()) {
                assertInstanceOf<Lc.Content<LocalNetworkSharingUiState>>(this)
                val lanSharingAllowed = value.lanSharingEnabled

                // Port should be what we expect and be selected
                assertEquals(allowLan, lanSharingAllowed)
            }
        }
    }
}
