package net.mullvad.mullvadvpn.feature.location.impl

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.HopSelection
import net.mullvad.mullvadvpn.lib.model.MultihopRelayListType
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints
import net.mullvad.mullvadvpn.lib.model.communication.CustomListAction
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.lib.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.lib.repository.RelayListRepository
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.lib.usecase.FilterChip
import net.mullvad.mullvadvpn.lib.usecase.FilterChipUseCase
import net.mullvad.mullvadvpn.lib.usecase.HopSelectionUseCase
import net.mullvad.mullvadvpn.lib.usecase.ModelOwnership
import net.mullvad.mullvadvpn.lib.usecase.ModifyMultihopUseCase
import net.mullvad.mullvadvpn.lib.usecase.MultihopChange
import net.mullvad.mullvadvpn.lib.usecase.SelectSinglehopUseCase
import net.mullvad.mullvadvpn.lib.usecase.customlists.CustomListActionUseCase
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class SelectLocationViewModelTest {

    private val mockRelayListFilterRepository: RelayListFilterRepository = mockk()
    private val mockCustomListActionUseCase: CustomListActionUseCase = mockk(relaxed = true)
    private val mockRelayListRepository: RelayListRepository = mockk()
    private val mockWireguardConstraintsRepository: WireguardConstraintsRepository = mockk()
    private val mockFilterChipUseCase: FilterChipUseCase = mockk()
    private val mockSettingsRepository: SettingsRepository = mockk()
    private val mockSelectSinglehopUseCase: SelectSinglehopUseCase = mockk()
    private val mockModifyMultihopUseCase: ModifyMultihopUseCase = mockk()
    private val mockHopSelectionUseCase: HopSelectionUseCase = mockk()
    private val mockConnectionProxy: ConnectionProxy = mockk()

    private val relayListScrollConnection: RelayListScrollConnection = RelayListScrollConnection()

    private lateinit var viewModel: SelectLocationViewModel

    private val selectedRelayItemFlow = MutableStateFlow<HopSelection>(HopSelection.Single(null))
    private val wireguardConstraints = MutableStateFlow<WireguardConstraints>(mockk(relaxed = true))
    private val filterChips = MutableStateFlow<List<FilterChip>>(emptyList())
    private val relayList = MutableStateFlow<List<RelayItem.Location.Country>>(emptyList())
    private val settings = MutableStateFlow<Settings>(mockk(relaxed = true))

    @BeforeEach
    fun setup() {

        every { mockWireguardConstraintsRepository.wireguardConstraints } returns
            wireguardConstraints
        every { mockFilterChipUseCase(any()) } returns filterChips
        every { mockRelayListRepository.relayList } returns relayList
        every { mockSettingsRepository.settingsUpdates } returns settings
        every { mockConnectionProxy.tunnelState } returns flowOf(mockk())
        every { mockHopSelectionUseCase() } returns selectedRelayItemFlow

        mockkStatic(RELAY_LIST_EXTENSIONS)
        mockkStatic(RELAY_ITEM_EXTENSIONS)
        mockkStatic(CUSTOM_LIST_EXTENSIONS)
        viewModel =
            SelectLocationViewModel(
                relayListFilterRepository = mockRelayListFilterRepository,
                customListActionUseCase = mockCustomListActionUseCase,
                relayListRepository = mockRelayListRepository,
                filterChipUseCase = mockFilterChipUseCase,
                wireguardConstraintsRepository = mockWireguardConstraintsRepository,
                settingsRepository = mockSettingsRepository,
                modifyMultihopUseCase = mockModifyMultihopUseCase,
                selectSingleUseCase = mockSelectSinglehopUseCase,
                hopSelectionUseCase = mockHopSelectionUseCase,
                connectionProxy = mockConnectionProxy,
                relayListScrollConnection = relayListScrollConnection,
            )
    }

    @AfterEach
    fun teardown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `initial state should be correct`() = runTest {
        assertIs<Lc.Loading<Unit>>(viewModel.uiState.value)
    }

    @Test
    fun `on modifyMultihop when relay list type is exit call uiSideEffect should emit CloseScreen and connect`() =
        runTest {
            // Arrange
            val mockRelayItem: RelayItem.Location.Country = mockk()
            val relayItemId: GeoLocationId.Country = mockk(relaxed = true)
            val multihopChange: MultihopChange = MultihopChange.Exit(mockRelayItem)
            every { mockRelayItem.id } returns relayItemId
            every { mockRelayItem.active } returns true
            coEvery { mockModifyMultihopUseCase.invoke(multihopChange) } returns Unit.right()

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.modifyMultihop(mockRelayItem, MultihopRelayListType.EXIT)
                // Await an empty item
                assertEquals(SelectLocationSideEffect.CloseScreen, awaitItem())
                coVerify { mockModifyMultihopUseCase.invoke(multihopChange) }
            }
        }

    @Test
    fun `on selectRelay when relay list type is entry call uiSideEffect should switch relay list type to exit`() =
        runTest {
            // Arrange
            val mockRelayItem: RelayItem.Location.Country = mockk()
            val relayItemId: GeoLocationId.Country = mockk(relaxed = true)
            val multihopChange = MultihopChange.Entry(mockRelayItem)
            every { mockRelayItem.active } returns true
            every { mockRelayItem.id } returns relayItemId
            coEvery { mockModifyMultihopUseCase.invoke(multihopChange) } returns Unit.right()

            // Act, Assert
            viewModel.uiState.test {
                awaitItem() // Default value
                viewModel.selectRelayList(MultihopRelayListType.ENTRY)
                // Assert relay list type is entry
                val firstState = awaitItem()
                assertIs<Lc.Content<SelectLocationUiState>>(firstState)
                assertEquals(MultihopRelayListType.ENTRY, firstState.value.multihopListSelection)
                // Select entry
                viewModel.modifyMultihop(mockRelayItem, MultihopRelayListType.ENTRY)
                // Assert relay list type is exit
                val secondState = awaitItem()
                assertIs<Lc.Content<SelectLocationUiState>>(secondState)
                assertEquals(MultihopRelayListType.EXIT, secondState.value.multihopListSelection)
                coVerify { mockModifyMultihopUseCase.invoke(multihopChange) }
            }
        }

    @Test
    fun `removeOwnerFilter should invoke use case with Constraint Any Ownership`() = runTest {
        // Arrange
        val mockSelectedProviders: Constraint<Providers> = mockk()
        every { mockRelayListFilterRepository.selectedProviders } returns
            MutableStateFlow(mockSelectedProviders)
        coEvery { mockRelayListFilterRepository.updateSelectedOwnership(Constraint.Any) } returns
            Unit.right()

        // Act
        viewModel.removeOwnerFilter()
        // Assert
        coVerify { mockRelayListFilterRepository.updateSelectedOwnership(Constraint.Any) }
    }

    @Test
    fun `removeProviderFilter should invoke use case with Constraint Any Provider`() = runTest {
        // Arrange
        val mockSelectedOwnership: Constraint<Ownership> = mockk()
        every { mockRelayListFilterRepository.selectedOwnership } returns
            MutableStateFlow(mockSelectedOwnership)
        coEvery { mockRelayListFilterRepository.updateSelectedProviders(Constraint.Any) } returns
            Unit.right()

        // Act
        viewModel.removeProviderFilter()
        // Assert
        coVerify { mockRelayListFilterRepository.updateSelectedProviders(Constraint.Any) }
    }

    @Test
    fun `when perform action is called should call custom list use case`() {
        // Arrange
        val action: CustomListAction = mockk()

        // Act
        viewModel.performAction(action)

        // Assert
        coVerify { mockCustomListActionUseCase(action) }
    }

    @Test
    fun `given entry blocked should filters if in entry list`() = runTest {
        // Arrange
        val mockSettings = mockk<Settings>(relaxed = true)
        settings.value = mockSettings
        every { mockSettings.tunnelOptions.daitaSettings.enabled } returns true
        every { mockSettings.tunnelOptions.daitaSettings.directOnly } returns false
        every {
            mockSettings.relaySettings.relayConstraints.wireguardConstraints.isMultihopEnabled
        } returns true
        val expectedFilters = listOf(FilterChip.Quic, FilterChip.Daita)
        filterChips.value = expectedFilters

        // Act, Assert
        viewModel.uiState.test {
            awaitItem() // Initial state
            viewModel.selectRelayList(MultihopRelayListType.ENTRY)
            val state = awaitItem()
            assertIs<Lc.Content<SelectLocationUiState>>(state)
            assertLists(expectedFilters, state.value.filterChips)
        }
    }

    @Test
    fun `given entry blocked should emit filters if in exit list`() = runTest {
        // Arrange
        val mockSettings = mockk<Settings>(relaxed = true)
        val expectedFilters = listOf(FilterChip.Ownership(ModelOwnership.MullvadOwned))
        settings.value = mockSettings
        filterChips.value = expectedFilters
        every { mockSettings.tunnelOptions.daitaSettings.enabled } returns true
        every { mockSettings.tunnelOptions.daitaSettings.directOnly } returns false
        every {
            mockSettings.relaySettings.relayConstraints.wireguardConstraints.isMultihopEnabled
        } returns true

        // Act, Assert
        viewModel.uiState.test {
            viewModel.selectRelayList(MultihopRelayListType.EXIT)
            val state = awaitItem()
            assertIs<Lc.Content<SelectLocationUiState>>(state)
            assertLists(expectedFilters, state.value.filterChips)
        }
    }

    companion object {
        private const val RELAY_LIST_EXTENSIONS =
            "net.mullvad.mullvadvpn.lib.common.util.relaylist.RelayListExtensionsKt"
        private const val RELAY_ITEM_EXTENSIONS =
            "net.mullvad.mullvadvpn.lib.common.util.relaylist.RelayItemExtensionsKt"
        private const val CUSTOM_LIST_EXTENSIONS =
            "net.mullvad.mullvadvpn.lib.common.util.relaylist.CustomListExtensionsKt"
    }
}
