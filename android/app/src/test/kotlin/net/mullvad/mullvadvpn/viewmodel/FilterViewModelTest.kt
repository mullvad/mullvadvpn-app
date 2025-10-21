package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import arrow.core.right
import com.ramcosta.composedestinations.generated.navargs.toSavedStateHandle
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.screen.FilterNavArgs
import net.mullvad.mullvadvpn.compose.state.toConstraintProviders
import net.mullvad.mullvadvpn.compose.state.toOwnershipConstraint
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.assertSet
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.usecase.ProviderToOwnershipsUseCase
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class FilterViewModelTest {
    private val mockProvidersOwnershipUseCase: ProviderToOwnershipsUseCase = mockk(relaxed = true)
    private val mockRelayListFilterRepository: RelayListFilterRepository = mockk()
    private lateinit var viewModel: FilterViewModel
    private val selectedOwnership =
        MutableStateFlow<Constraint<Ownership>>(Constraint.Only(Ownership.MullvadOwned))
    private val dummyListOfAllProviders =
        mapOf(
            ProviderId("31173") to setOf(Ownership.MullvadOwned),
            ProviderId("100TB") to setOf(Ownership.Rented),
            ProviderId("Blix") to setOf(Ownership.MullvadOwned),
            ProviderId("Creanova") to setOf(Ownership.MullvadOwned),
            ProviderId("DataPacket") to setOf(Ownership.Rented, Ownership.MullvadOwned),
            ProviderId("HostRoyale") to setOf(Ownership.Rented),
            ProviderId("hostuniversal") to setOf(Ownership.Rented),
            ProviderId("iRegister") to setOf(Ownership.Rented),
            ProviderId("M247") to setOf(Ownership.Rented),
            ProviderId("Makonix") to setOf(Ownership.Rented),
            ProviderId("PrivateLayer") to setOf(Ownership.Rented),
            ProviderId("ptisp") to setOf(Ownership.Rented),
            ProviderId("Qnax") to setOf(Ownership.Rented),
            ProviderId("Quadranet") to setOf(Ownership.Rented),
            ProviderId("techfutures") to setOf(Ownership.Rented),
            ProviderId("Tzulo") to setOf(Ownership.Rented),
            ProviderId("xtom") to setOf(Ownership.Rented),
        )
    private val mockSelectedProviders: Providers =
        setOf(ProviderId("31173"), ProviderId("Blix"), ProviderId("Creanova"))

    @BeforeEach
    fun setup() {
        every { mockRelayListFilterRepository.selectedOwnership(any()) } returns selectedOwnership
        every { mockRelayListFilterRepository.selectedOwnership(any()) } returns selectedOwnership
        every { mockProvidersOwnershipUseCase() } returns flowOf(dummyListOfAllProviders)
        every { mockRelayListFilterRepository.selectedProviders(any()) } returns
            MutableStateFlow(Constraint.Only(mockSelectedProviders))
        every { mockRelayListFilterRepository.selectedProviders(any()) } returns
            MutableStateFlow(Constraint.Only(mockSelectedProviders))
        viewModel =
            FilterViewModel(
                providerToOwnershipsUseCase = mockProvidersOwnershipUseCase,
                relayListFilterRepository = mockRelayListFilterRepository,
                savedStateHandle =
                    FilterNavArgs(filterType = RelayListType.Single).toSavedStateHandle(),
            )
    }

    @AfterEach
    fun teardown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `setSelectedOwnership with Rented should emit uiState where selectedOwnership is Rented`() =
        runTest {
            // Arrange
            val mockOwnership = Ownership.Rented
            // Assert
            viewModel.uiState.test {
                assertEquals(Constraint.Only(Ownership.MullvadOwned), awaitItem().selectedOwnership)
                viewModel.setSelectedOwnership(Constraint.Only(mockOwnership))
                assertEquals(Constraint.Only(mockOwnership), awaitItem().selectedOwnership)
            }
        }

    @Test
    fun `setSelectionProvider should emit uiState where selectedProviders include the selected provider`() =
        runTest {
            // Arrange
            val mockSelectedProvidersList = ProviderId("ptisp")
            // Assert
            viewModel.uiState.test {
                assertSet(mockSelectedProviders, awaitItem().selectedProviders.getOrNull()!!)
                viewModel.setSelectedProvider(true, mockSelectedProvidersList)
                assertSet(
                    setOf(mockSelectedProvidersList) + mockSelectedProviders,
                    awaitItem().selectedProviders.getOrNull()!!,
                )
            }
        }

    @Test
    fun `setAllProvider with true should emit uiState with selectedProviders includes all providers`() =
        runTest {
            // Act
            viewModel.setAllProviders(true)
            // Assert
            viewModel.uiState.test {
                val state = awaitItem()
                assertEquals(Constraint.Any, state.selectedProviders)
            }
        }

    @Test
    fun `onApplyButtonClicked should invoke updateOwnershipAndProviderFilter on RelayListFilterUseCase`() =
        runTest {
            // Arrange
            val mockOwnership = Ownership.MullvadOwned.toOwnershipConstraint()
            val mockSelectedProviders =
                mockSelectedProviders.toConstraintProviders(dummyListOfAllProviders.keys)
            coEvery {
                mockRelayListFilterRepository.updateSelectedOwnershipAndProviderFilter(
                    mockOwnership,
                    mockSelectedProviders,
                    RelayListType.Single,
                )
            } returns Unit.right()

            // Act
            viewModel.onApplyButtonClicked()

            // Assert
            coVerify {
                mockRelayListFilterRepository.updateSelectedOwnershipAndProviderFilter(
                    mockOwnership,
                    mockSelectedProviders,
                    RelayListType.Single,
                )
            }
        }

    @Test
    fun `ensure that providers with multiple ownership are only returned once`() = runTest {
        // Arrange
        val expectedProviderList = dummyListOfAllProviders.keys

        // Assert
        viewModel.uiState.test {
            val state = awaitItem()
            assertSet(expectedProviderList, state.allProviders)
        }
    }

    @Test
    fun `ensure that providers are sorted by name`() = runTest {
        // Assert
        viewModel.uiState.test {
            val state = awaitItem()
            assertSet(state.allProviders, state.allProviders)
            assertEquals(state.selectableProviders.sorted(), state.selectableProviders)
        }
    }
}
