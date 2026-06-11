package net.mullvad.mullvadvpn.feature.multihop.impl

import app.cash.turbine.test
import arrow.core.Either
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository
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
                isModal = false,
                wireguardConstraintsRepository = mockWireguardConstraintsRepository,
            )
    }

    @Test
    fun `when multihop always is true state should return multihop always`() = runTest {
        // Arrange
        wireguardConstraints.value =
            WireguardConstraints(
                multihop = MultihopMode.ALWAYS,
                entryLocation = Constraint.Any,
                ipVersion = Constraint.Any,
                entryOwnership = Constraint.Any,
                entryProviders = Constraint.Any,
            )

        // Act, Assert
        multihopViewModel.uiState.test {
            val item = awaitItem()
            assertIs<Lc.Content<MultihopUiState>>(item)
            assertEquals(MultihopUiState(mode = MultihopMode.ALWAYS), item.value)
        }
    }

    @Test
    fun `when set multihop is called should call repository set multihop`() = runTest {
        // Arrange
        coEvery { mockWireguardConstraintsRepository.setMultihop(any()) } returns Either.Right(Unit)

        // Act
        multihopViewModel.setMultihopMode(MultihopMode.NEVER)

        // Assert
        coVerify { mockWireguardConstraintsRepository.setMultihop(MultihopMode.NEVER) }
    }
}
