package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import arrow.core.Either
import com.ramcosta.composedestinations.generated.navargs.toSavedStateHandle
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.screen.MultihopNavArgs
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class MultihopViewModelTest {

    private val mockWireguardConstraintsRepository: WireguardConstraintsRepository = mockk()

    private val wireguardConstraints = MutableStateFlow<WireguardConstraints>(mockk(relaxed = true))

    private lateinit var multihopViewModel: MultihopViewModel

    @BeforeEach
    fun setUp() {
        every { mockWireguardConstraintsRepository.wireguardConstraints } returns
            wireguardConstraints

        multihopViewModel =
            MultihopViewModel(
                wireguardConstraintsRepository = mockWireguardConstraintsRepository,
                savedStateHandle = MultihopNavArgs().toSavedStateHandle(),
            )
    }

    @Test
    fun `default state should be multihop disabled`() {
        assertEquals(false, multihopViewModel.uiState.value.enable)
    }

    @Test
    fun `when multihop enabled is true state should return multihop enabled true`() = runTest {
        // Arrange
        wireguardConstraints.value =
            WireguardConstraints(
                isMultihopEnabled = true,
                entryLocation = Constraint.Any,
                port = Constraint.Any,
                ipVersion = Constraint.Any,
            )

        // Act, Assert
        multihopViewModel.uiState.test { assertEquals(MultihopUiState(true), awaitItem()) }
    }

    @Test
    fun `when set multihop is called should call repository set multihop`() = runTest {
        // Arrange
        coEvery { mockWireguardConstraintsRepository.setMultihop(any()) } returns Either.Right(Unit)

        // Act
        multihopViewModel.setMultihop(true)

        // Assert
        coVerify { mockWireguardConstraintsRepository.setMultihop(true) }
    }
}
