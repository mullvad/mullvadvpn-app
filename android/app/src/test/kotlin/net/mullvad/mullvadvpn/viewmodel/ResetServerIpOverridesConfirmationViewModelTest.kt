package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import io.mockk.verify
import kotlin.test.assertEquals
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.RelayOverride
import net.mullvad.mullvadvpn.repository.RelayOverridesRepository
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class ResetServerIpOverridesConfirmationViewModelTest {
    private lateinit var viewModel: ResetServerIpOverridesConfirmationViewModel

    private val mockRelayOverridesRepository: RelayOverridesRepository = mockk()
    private val relayOverrides = MutableStateFlow<List<RelayOverride>?>(null)

    @BeforeEach
    fun setup() {
        coEvery { mockRelayOverridesRepository.relayOverrides } returns relayOverrides

        viewModel =
            ResetServerIpOverridesConfirmationViewModel(
                relayOverridesRepository = mockRelayOverridesRepository,
            )
    }

    @AfterEach
    fun teardown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `successful clear of override should result in side effect`() = runTest {
        every { mockRelayOverridesRepository.clearAllOverrides() } returns Unit
        viewModel.uiSideEffect.test {
            viewModel.clearAllOverrides()
            assertEquals(
                ResetServerIpOverridesConfirmationUiSideEffect.OverridesCleared,
                awaitItem()
            )
        }
    }

    @Test
    fun `clear overrides should invoke repository`() = runTest {
        every { mockRelayOverridesRepository.clearAllOverrides() } returns Unit
        viewModel.clearAllOverrides()
        verify { mockRelayOverridesRepository.clearAllOverrides() }
    }
}
