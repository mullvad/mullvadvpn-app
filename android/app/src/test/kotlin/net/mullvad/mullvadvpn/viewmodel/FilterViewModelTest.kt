package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
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
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Providers
import net.mullvad.mullvadvpn.relaylist.Provider
import net.mullvad.mullvadvpn.usecase.RelayListFilterUseCase
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class FilterViewModelTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()
    private val mockRelayListFilterUseCase: RelayListFilterUseCase = mockk(relaxed = true)
    private lateinit var viewModel: FilterViewModel
    private val selectedOwnership =
        MutableStateFlow<Constraint<Ownership>>(Constraint.Only(Ownership.MullvadOwned))
    private val dummyListOfAllProviders =
        listOf(
            Provider("31173", true),
            Provider("100TB", false),
            Provider("Blix", true),
            Provider("Creanova", true),
            Provider("DataPacket", false),
            Provider("HostRoyale", false),
            Provider("hostuniversal", false),
            Provider("iRegister", false),
            Provider("M247", false),
            Provider("Makonix", false),
            Provider("PrivateLayer", false),
            Provider("ptisp", false),
            Provider("Qnax", false),
            Provider("Quadranet", false),
            Provider("techfutures", false),
            Provider("Tzulo", false),
            Provider("xtom", false)
        )
    private val mockSelectedProviders: List<Provider> =
        listOf(Provider("31173", true), Provider("Blix", true), Provider("Creanova", true))

    @Before
    fun setup() {
        every { mockRelayListFilterUseCase.selectedOwnership() } returns selectedOwnership
        every { mockRelayListFilterUseCase.availableProviders() } returns
            flowOf(dummyListOfAllProviders)
        every { mockRelayListFilterUseCase.selectedProviders() } returns
            flowOf(Constraint.Only(Providers(mockSelectedProviders.map { it.name }.toHashSet())))
        viewModel = FilterViewModel(mockRelayListFilterUseCase)
    }

    @After
    fun teardown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun testSetSelectedOwnership() = runTest {
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
    fun testSetSelectedProvider() = runTest {
        // Arrange
        val mockSelectedProvidersList = Provider("ptisp", false)
        // Assert
        viewModel.uiState.test {
            assertLists(awaitItem().selectedProviders, mockSelectedProviders)
            viewModel.setSelectedProvider(true, mockSelectedProvidersList)
            assertLists(
                listOf(mockSelectedProvidersList) + mockSelectedProviders,
                awaitItem().selectedProviders
            )
        }
    }

    @Test
    fun testSetAllProviders() = runTest {
        // Arrange
        val mockProvidersList = dummyListOfAllProviders
        // Act
        viewModel.setAllProviders(true)
        // Assert
        viewModel.uiState.test {
            val state = awaitItem()
            assertEquals(mockProvidersList, state.selectedProviders)
        }
    }

    @Test
    fun testOnApplyButtonClicked() = runTest {
        // Arrange
        val mockOwnership = Ownership.MullvadOwned.toOwnershipConstraint()
        val mockSelectedProviders =
            mockSelectedProviders.toConstraintProviders(dummyListOfAllProviders)
        // Act
        viewModel.onApplyButtonClicked()
        // Assert
        coVerify {
            mockRelayListFilterUseCase.updateOwnershipAndProviderFilter(
                mockOwnership,
                mockSelectedProviders
            )
        }
    }
}
