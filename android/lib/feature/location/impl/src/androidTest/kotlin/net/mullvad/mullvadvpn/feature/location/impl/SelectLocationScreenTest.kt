package net.mullvad.mullvadvpn.feature.location.impl

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.hasAnyAncestor
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import io.mockk.verify
import kotlinx.coroutines.flow.MutableStateFlow
import net.mullvad.mullvadvpn.feature.location.api.LocationBottomSheetState
import net.mullvad.mullvadvpn.feature.location.impl.bottomsheet.LocationBottomSheetUiState
import net.mullvad.mullvadvpn.feature.location.impl.bottomsheet.LocationBottomSheetViewModel
import net.mullvad.mullvadvpn.feature.location.impl.bottomsheet.SetAsState
import net.mullvad.mullvadvpn.feature.location.impl.data.DUMMY_RELAY_COUNTRIES
import net.mullvad.mullvadvpn.feature.location.impl.data.DUMMY_RELAY_ITEM_CUSTOM_LISTS
import net.mullvad.mullvadvpn.feature.location.impl.data.createSimpleRelayListItemList
import net.mullvad.mullvadvpn.feature.location.impl.list.SelectLocationListUiState
import net.mullvad.mullvadvpn.feature.location.impl.list.SelectLocationListViewModel
import net.mullvad.mullvadvpn.feature.location.impl.util.onNodeTextAndAncestorTag
import net.mullvad.mullvadvpn.feature.location.impl.util.performLongClick
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.Lce
import net.mullvad.mullvadvpn.lib.model.HopSelection
import net.mullvad.mullvadvpn.lib.model.MultihopRelayListType
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.RelayListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.tag.GEOLOCATION_ITEM_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.RECENT_CELL_TEST_TAG
import net.mullvad.mullvadvpn.screen.test.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.screen.test.setContentWithTheme
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension
import org.koin.core.context.loadKoinModules
import org.koin.core.module.dsl.viewModel
import org.koin.dsl.module

@OptIn(ExperimentalTestApi::class)
class SelectLocationScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    private val listViewModel: SelectLocationListViewModel = mockk(relaxed = true)
    private val bottomSheetViewModel: LocationBottomSheetViewModel = mockk(relaxed = true)

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
        loadKoinModules(
            module {
                viewModel { listViewModel }
                viewModel { bottomSheetViewModel }
            }
        )
        every { listViewModel.uiState } returns MutableStateFlow(Lce.Loading(Unit))
        every { bottomSheetViewModel.uiState } returns MutableStateFlow(Lc.Loading(Unit))
    }

    @AfterEach
    fun teardown() {
        unmockkAll()
    }

    private fun ComposeContext.initScreen(
        state: Lc<Unit, SelectLocationUiState> = Lc.Loading(Unit),
        onSelectHop: (item: RelayItem) -> Unit = {},
        onUpdateBottomSheetState: (LocationBottomSheetState) -> Unit = {},
        onModifyMultihop: (RelayItem, MultihopRelayListType) -> Unit = { _, _ -> },
        onSearchClick: (RelayListType) -> Unit = {},
        onBackClick: () -> Unit = {},
        onFilterClick: () -> Unit = {},
        onCreateCustomList: () -> Unit = {},
        onEditCustomLists: () -> Unit = {},
        removeOwnershipFilter: () -> Unit = {},
        removeProviderFilter: () -> Unit = {},
        onSelectRelayList: (MultihopRelayListType) -> Unit = {},
        openDaitaSettings: () -> Unit = {},
        onRecentsToggleEnableClick: () -> Unit = {},
        onRefreshRelayList: () -> Unit = {},
        onScrollToItem: (ScrollEvent) -> Unit = {},
        toggleMultihop: (Boolean) -> Unit = {},
    ) {

        setContentWithTheme {
            SelectLocationScreen(
                state = state,
                navigateToBottomSheet = onUpdateBottomSheetState,
                onSelectSinglehop = onSelectHop,
                onModifyMultihop = onModifyMultihop,
                onSearchClick = onSearchClick,
                onBackClick = onBackClick,
                onFilterClick = onFilterClick,
                onEditCustomLists = onEditCustomLists,
                onCreateCustomList = onCreateCustomList,
                removeOwnershipFilter = removeOwnershipFilter,
                removeProviderFilter = removeProviderFilter,
                onSelectRelayList = onSelectRelayList,
                openDaitaSettings = openDaitaSettings,
                onRecentsToggleEnableClick = onRecentsToggleEnableClick,
                onRefreshRelayList = onRefreshRelayList,
                scrollToItem = onScrollToItem,
                toggleMultihop = toggleMultihop,
            )
        }
    }

    @Test
    fun testShowRelayListState() = composeExtension.use {
        // Arrange
        every { listViewModel.uiState } returns
            MutableStateFlow(
                Lce.Content(
                    SelectLocationListUiState(
                        relayListType = RelayListType.Single,
                        relayListItems =
                            DUMMY_RELAY_COUNTRIES.map {
                                RelayListItem.GeoLocationItem(
                                    item = it,
                                    itemPosition = Position.Single,
                                    hierarchy = Hierarchy.Parent,
                                )
                            },
                    )
                )
            )
        initScreen(
            state =
                Lc.Content(
                    SelectLocationUiState(
                        filterChips = emptyList(),
                        multihopListSelection = MultihopRelayListType.EXIT,
                        isSearchButtonEnabled = true,
                        isFilterButtonEnabled = true,
                        isRecentsEnabled = true,
                        hopSelection = HopSelection.Single(null),
                        tunnelErrorStateCause = null,
                    )
                )
        )

        // Assert
        onNodeWithText("Relay Country 1").assertExists()
        onNodeWithText("Relay City 1").assertDoesNotExist()
        onNodeWithText("Relay host 1").assertDoesNotExist()
        onNodeWithText("Relay Country 2").assertExists()
        onNodeWithText("Relay City 2").assertDoesNotExist()
        onNodeWithText("Relay host 2").assertDoesNotExist()
    }

    @Test
    fun customListFooterShouldShowEmptyTextWhenNoCustomList() = composeExtension.use {
        // Arrange
        every { listViewModel.uiState } returns
            MutableStateFlow(
                Lce.Content(
                    SelectLocationListUiState(
                        relayListItems = listOf(RelayListItem.CustomListFooter(false)),
                        relayListType = RelayListType.Single,
                    )
                )
            )
        initScreen(
            state =
                Lc.Content(
                    SelectLocationUiState(
                        filterChips = emptyList(),
                        multihopListSelection = MultihopRelayListType.EXIT,
                        isSearchButtonEnabled = true,
                        isFilterButtonEnabled = true,
                        isRecentsEnabled = true,
                        hopSelection = HopSelection.Single(null),
                        tunnelErrorStateCause = null,
                    )
                )
        )

        // Assert
        onNodeWithText(CUSTOM_LISTS_EMPTY_TEXT).assertExists()
    }

    @Test
    fun whenCustomListIsClickedShouldCallOnSelectHop() = composeExtension.use {
        // Arrange
        val customList = DUMMY_RELAY_ITEM_CUSTOM_LISTS[0]
        every { listViewModel.uiState } returns
            MutableStateFlow(
                Lce.Content(
                    SelectLocationListUiState(
                        relayListItems = listOf(RelayListItem.CustomListItem(customList)),
                        relayListType = RelayListType.Single,
                    )
                )
            )
        val mockedOnSelectHop: (RelayItem) -> Unit = mockk(relaxed = true)
        initScreen(
            state =
                Lc.Content(
                    SelectLocationUiState(
                        filterChips = emptyList(),
                        multihopListSelection = MultihopRelayListType.EXIT,
                        isSearchButtonEnabled = true,
                        isFilterButtonEnabled = true,
                        isRecentsEnabled = true,
                        hopSelection = HopSelection.Single(null),
                        tunnelErrorStateCause = null,
                    )
                ),
            onSelectHop = mockedOnSelectHop,
        )

        // Act
        onNodeWithText(customList.name).performClick()

        // Assert
        verify { mockedOnSelectHop(customList) }
    }

    @Test
    fun whenRecentIsClickedShouldCallOnSelectHop() = composeExtension.use {
        // Arrange
        val recent = DUMMY_RELAY_COUNTRIES[0]
        every { listViewModel.uiState } returns
            MutableStateFlow(
                Lce.Content(
                    SelectLocationListUiState(
                        relayListItems = listOf(RelayListItem.RecentListItem(item = recent)),
                        relayListType = RelayListType.Single,
                    )
                )
            )
        val mockedOnSelectHop: (RelayItem) -> Unit = mockk(relaxed = true)
        initScreen(
            state =
                Lc.Content(
                    SelectLocationUiState(
                        filterChips = emptyList(),
                        multihopListSelection = MultihopRelayListType.EXIT,
                        isSearchButtonEnabled = true,
                        isFilterButtonEnabled = true,
                        isRecentsEnabled = true,
                        hopSelection = HopSelection.Single(null),
                        tunnelErrorStateCause = null,
                    )
                ),
            onSelectHop = mockedOnSelectHop,
        )

        // Act
        onNodeWithText(recent.name).performClick()

        // Assert
        verify { mockedOnSelectHop(recent) }
    }

    @Test
    fun ensureCustomListLongClickWorks() = composeExtension.use {
        // Arrange
        val customList = DUMMY_RELAY_ITEM_CUSTOM_LISTS[0]
        every { listViewModel.uiState } returns
            MutableStateFlow(
                Lce.Content(
                    SelectLocationListUiState(
                        relayListItems = listOf(RelayListItem.CustomListItem(item = customList)),
                        relayListType = RelayListType.Single,
                    )
                )
            )
        every { bottomSheetViewModel.uiState } returns
            MutableStateFlow(
                Lc.Content(
                    LocationBottomSheetUiState.CustomList(
                        item = customList,
                        setAsExitState = SetAsState.HIDDEN,
                        setAsEntryState = SetAsState.ENABLED,
                        canDisableMultihop = false,
                    )
                )
            )
        val mockedOnSelectHop: (RelayItem) -> Unit = mockk(relaxed = true)
        initScreen(
            state =
                Lc.Content(
                    SelectLocationUiState(
                        filterChips = emptyList(),
                        multihopListSelection = MultihopRelayListType.EXIT,
                        isSearchButtonEnabled = true,
                        isFilterButtonEnabled = true,
                        isRecentsEnabled = true,
                        hopSelection = HopSelection.Single(null),
                        tunnelErrorStateCause = null,
                    )
                ),
            onSelectHop = mockedOnSelectHop,
        )

        // Act
        onNodeWithText(customList.name).assertExists().performLongClick()
    }

    @Test
    fun ensureLocationLongClickWorks() = composeExtension.use {
        // Arrange
        val relayItem = DUMMY_RELAY_COUNTRIES[0] as RelayItem.Location
        every { listViewModel.uiState } returns
            MutableStateFlow(
                Lce.Content(
                    SelectLocationListUiState(
                        relayListItems =
                            listOf(
                                RelayListItem.GeoLocationItem(
                                    relayItem,
                                    itemPosition = Position.Single,
                                    hierarchy = Hierarchy.Parent,
                                )
                            ),
                        relayListType = RelayListType.Single,
                    )
                )
            )
        every { bottomSheetViewModel.uiState } returns
            MutableStateFlow(
                Lc.Content(
                    LocationBottomSheetUiState.Location(
                        item = relayItem,
                        customLists = emptyList(),
                        setAsExitState = SetAsState.HIDDEN,
                        setAsEntryState = SetAsState.ENABLED,
                        canDisableMultihop = false,
                    )
                )
            )
        val mockedOnSelectHop: (RelayItem) -> Unit = mockk(relaxed = true)
        initScreen(
            state =
                Lc.Content(
                    SelectLocationUiState(
                        filterChips = emptyList(),
                        multihopListSelection = MultihopRelayListType.EXIT,
                        isSearchButtonEnabled = true,
                        isFilterButtonEnabled = true,
                        isRecentsEnabled = true,
                        hopSelection = HopSelection.Single(null),
                        tunnelErrorStateCause = null,
                    )
                ),
            onSelectHop = mockedOnSelectHop,
        )

        // Act
        onNodeWithText(relayItem.name).assertExists().performLongClick()
    }

    @Test
    fun whenOpeningScreenAndRecentsEnabledShouldScrollToTheSelectedRecent() {
        composeExtension.use {
            // Arrange
            val selectableItem = DUMMY_RELAY_COUNTRIES[3].relays.last()
            every { listViewModel.uiState } returns
                MutableStateFlow(
                    Lce.Content(
                        SelectLocationListUiState(
                            relayListItems =
                                createSimpleRelayListItemList(
                                    recentItems = listOf(selectableItem),
                                    customListItem = DUMMY_RELAY_ITEM_CUSTOM_LISTS,
                                    locationItems = DUMMY_RELAY_COUNTRIES,
                                    selectedItem = selectableItem.id,
                                ),
                            relayListType = RelayListType.Single,
                        )
                    )
                )
            initScreen(
                state =
                    Lc.Content(
                        SelectLocationUiState(
                            filterChips = emptyList(),
                            multihopListSelection = MultihopRelayListType.EXIT,
                            isSearchButtonEnabled = true,
                            isFilterButtonEnabled = true,
                            isRecentsEnabled = true,
                            hopSelection = HopSelection.Single(null),
                            tunnelErrorStateCause = null,
                        )
                    )
            )

            // Assert
            onNode(
                    hasText(selectableItem.name)
                        .and(hasAnyAncestor(hasTestTag(RECENT_CELL_TEST_TAG))),
                    useUnmergedTree = true,
                )
                .assertExists()
        }
    }

    @Test
    fun whenOpeningScreenAndRecentsDisabledShouldScrollToTheSelectedLocation() {
        composeExtension.use {
            // Arrange
            val selectableItem = DUMMY_RELAY_COUNTRIES[3].relays.last()
            every { listViewModel.uiState } returns
                MutableStateFlow(
                    Lce.Content(
                        SelectLocationListUiState(
                            relayListItems =
                                createSimpleRelayListItemList(
                                    customListItem = DUMMY_RELAY_ITEM_CUSTOM_LISTS,
                                    locationItems = DUMMY_RELAY_COUNTRIES,
                                    selectedItem = selectableItem.id,
                                ),
                            relayListType = RelayListType.Single,
                        )
                    )
                )
            initScreen(
                state =
                    Lc.Content(
                        SelectLocationUiState(
                            filterChips = emptyList(),
                            multihopListSelection = MultihopRelayListType.EXIT,
                            isSearchButtonEnabled = true,
                            isFilterButtonEnabled = true,
                            isRecentsEnabled = false,
                            hopSelection = HopSelection.Single(null),
                            tunnelErrorStateCause = null,
                        )
                    )
            )

            // Assert
            onNodeTextAndAncestorTag(
                    ancestorTag = GEOLOCATION_ITEM_TAG,
                    text = selectableItem.name,
                    useUnmergedTree = true,
                )
                .assertExists()
        }
    }

    @Test
    fun whenRelayListIsEmptyShouldShowEmptyState() = composeExtension.use {
        // Arrange
        every { listViewModel.uiState } returns
            MutableStateFlow(
                Lce.Content(
                    SelectLocationListUiState(
                        relayListType = RelayListType.Single,
                        relayListItems =
                            createSimpleRelayListItemList(
                                recentItems = emptyList(),
                                customListItem = emptyList(),
                                locationItems = emptyList(),
                                selectedItem = null,
                            ),
                    )
                )
            )
        initScreen(
            state =
                Lc.Content(
                    SelectLocationUiState(
                        filterChips = emptyList(),
                        multihopListSelection = MultihopRelayListType.EXIT,
                        isSearchButtonEnabled = false,
                        isFilterButtonEnabled = false,
                        isRecentsEnabled = true,
                        hopSelection = HopSelection.Single(null),
                        tunnelErrorStateCause = null,
                    )
                )
        )

        // Assert
        onNodeWithText(RELAY_LOCATIONS_EMPTY_TEXT_FIRST_LINE).assertExists()
        onNodeWithText(RELAY_LOCATIONS_EMPTY_TEXT_SECOND_LINE).assertExists()
    }

    companion object {
        private const val CUSTOM_LISTS_EMPTY_TEXT = "To create a custom list press the \"+\""
        private const val RELAY_LOCATIONS_EMPTY_TEXT_FIRST_LINE = "No matching servers found."
        private const val RELAY_LOCATIONS_EMPTY_TEXT_SECOND_LINE =
            "Please try changing your filters."
    }
}
