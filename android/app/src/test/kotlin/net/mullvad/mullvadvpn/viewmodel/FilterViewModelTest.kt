package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import arrow.core.right
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
import net.mullvad.mullvadvpn.compose.state.toConstraintProviders
import net.mullvad.mullvadvpn.compose.state.toOwnershipConstraint
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.usecase.AvailableProvidersUseCase
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class FilterViewModelTest {
    private val mockAvailableProvidersUseCase: AvailableProvidersUseCase = mockk(relaxed = true)
    private val mockRelayListFilterRepository: RelayListFilterRepository = mockk()
    private lateinit var viewModel: FilterViewModel
    private val selectedOwnership =
        MutableStateFlow<Constraint<Ownership>>(Constraint.Only(Ownership.MullvadOwned))
    private val dummyListOfAllProviders =
        listOf(
            Provider(ProviderId("31173"), setOf(Ownership.MullvadOwned)),
            Provider(ProviderId("100TB"), setOf(Ownership.Rented)),
            Provider(ProviderId("Blix"), setOf(Ownership.MullvadOwned)),
            Provider(ProviderId("Creanova"), setOf(Ownership.MullvadOwned)),
            Provider(ProviderId("DataPacket"), setOf(Ownership.Rented, Ownership.MullvadOwned)),
            Provider(ProviderId("HostRoyale"), setOf(Ownership.Rented)),
            Provider(ProviderId("hostuniversal"), setOf(Ownership.Rented)),
            Provider(ProviderId("iRegister"), setOf(Ownership.Rented)),
            Provider(ProviderId("M247"), setOf(Ownership.Rented)),
            Provider(ProviderId("Makonix"), setOf(Ownership.Rented)),
            Provider(ProviderId("PrivateLayer"), setOf(Ownership.Rented)),
            Provider(ProviderId("ptisp"), setOf(Ownership.Rented)),
            Provider(ProviderId("Qnax"), setOf(Ownership.Rented)),
            Provider(ProviderId("Quadranet"), setOf(Ownership.Rented)),
            Provider(ProviderId("techfutures"), setOf(Ownership.Rented)),
            Provider(ProviderId("Tzulo"), setOf(Ownership.Rented)),
            Provider(ProviderId("xtom"), setOf(Ownership.Rented)),
        )
    private val mockSelectedProviders: List<ProviderId> =
        listOf(ProviderId("31173"), ProviderId("Blix"), ProviderId("Creanova"))

    @BeforeEach
    fun setup() {
        every { mockRelayListFilterRepository.selectedOwnership } returns selectedOwnership
        every { mockAvailableProvidersUseCase() } returns flowOf(dummyListOfAllProviders)
        every { mockRelayListFilterRepository.selectedProviders } returns
            MutableStateFlow(Constraint.Only(Providers(mockSelectedProviders.toHashSet())))
        viewModel =
            FilterViewModel(
                availableProvidersUseCase = mockAvailableProvidersUseCase,
                relayListFilterRepository = mockRelayListFilterRepository,
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
                assertEquals(awaitItem().selectedOwnership, Ownership.MullvadOwned)
                viewModel.setSelectedOwnership(mockOwnership)
                assertEquals(mockOwnership, awaitItem().selectedOwnership)
            }
        }

    @Test
    fun `setSelectionProvider should emit uiState where selectedProviders include the selected provider`() =
        runTest {
            // Arrange
            val mockSelectedProvidersList = ProviderId("ptisp")
            // Assert
            viewModel.uiState.test {
                assertLists(awaitItem().selectedProviders, mockSelectedProviders)
                viewModel.setSelectedProvider(true, mockSelectedProvidersList)
                assertLists(
                    listOf(mockSelectedProvidersList) + mockSelectedProviders,
                    awaitItem().selectedProviders,
                )
            }
        }

    @Test
    fun `setAllProvider with true should emit uiState with selectedProviders includes all providers`() =
        runTest {
            // Arrange
            val mockProvidersList = dummyListOfAllProviders.map { it.providerId }
            // Act
            viewModel.setAllProviders(true)
            // Assert
            viewModel.uiState.test {
                val state = awaitItem()
                assertEquals(mockProvidersList, state.selectedProviders)
            }
        }

    @Test
    fun `onApplyButtonClicked should invoke updateOwnershipAndProviderFilter on RelayListFilterUseCase`() =
        runTest {
            // Arrange
            val mockOwnership = Ownership.MullvadOwned.toOwnershipConstraint()
            val mockSelectedProviders =
                mockSelectedProviders.toConstraintProviders(
                    dummyListOfAllProviders.map { it.providerId }
                )
            coEvery {
                mockRelayListFilterRepository.updateSelectedOwnershipAndProviderFilter(
                    mockOwnership,
                    mockSelectedProviders,
                )
            } returns Unit.right()

            // Act
            viewModel.onApplyButtonClicked()

            // Assert
            coVerify {
                mockRelayListFilterRepository.updateSelectedOwnershipAndProviderFilter(
                    mockOwnership,
                    mockSelectedProviders,
                )
            }
        }

    @Test
    fun `ensure that providers with multiple ownership are only returned once`() = runTest {
        // Arrange
        val expectedProviderList = dummyListOfAllProviders.map { it.providerId }

        // Assert
        viewModel.uiState.test {
            val state = awaitItem()
            assertLists(expectedProviderList, state.allProviders)
        }
    }
}
