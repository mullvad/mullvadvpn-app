package net.mullvad.mullvadvpn.compose.screen.location

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import io.mockk.verify
import kotlinx.coroutines.flow.MutableStateFlow
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.data.DUMMY_RELAY_COUNTRIES
import net.mullvad.mullvadvpn.compose.data.DUMMY_RELAY_ITEM_CUSTOM_LISTS
import net.mullvad.mullvadvpn.compose.data.createSimpleRelayListItemList
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.MultihopRelayListType
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.compose.state.SelectLocationListUiState
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.ItemPosition
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.RelayListItem
import net.mullvad.mullvadvpn.lib.ui.tag.GEOLOCATION_NAME_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.RECENT_NAME_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_CUSTOM_LIST_BOTTOM_SHEET_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_LOCATION_BOTTOM_SHEET_TEST_TAG
import net.mullvad.mullvadvpn.onNodeWithTagAndText
import net.mullvad.mullvadvpn.performLongClick
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.Lce
import net.mullvad.mullvadvpn.viewmodel.location.SelectLocationListViewModel
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

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
        loadKoinModules(module { viewModel { listViewModel } })
        every { listViewModel.uiState } returns MutableStateFlow(Lce.Loading(Unit))
    }

    @AfterEach
    fun teardown() {
        unmockkAll()
    }

    private fun ComposeContext.initScreen(
        state: Lc<Unit, SelectLocationUiState> = Lc.Loading(Unit),
        onSelectHop: (hop: Hop) -> Unit = {},
        onModifyMultihop: (RelayItem, MultihopRelayListType) -> Unit = { _, _ -> },
        onSearchClick: (RelayListType) -> Unit = {},
        onBackClick: () -> Unit = {},
        onFilterClick: () -> Unit = {},
        onCreateCustomList: (location: RelayItem.Location?) -> Unit = {},
        onEditCustomLists: () -> Unit = {},
        removeOwnershipFilter: () -> Unit = {},
        removeProviderFilter: () -> Unit = {},
        onAddLocationToList:
            (location: RelayItem.Location, customList: RelayItem.CustomList) -> Unit =
            { _, _ ->
            },
        onRemoveLocationFromList:
            (location: RelayItem.Location, customListId: CustomListId) -> Unit =
            { _, _ ->
            },
        onEditCustomListName: (RelayItem.CustomList) -> Unit = {},
        onEditLocationsCustomList: (RelayItem.CustomList) -> Unit = {},
        onDeleteCustomList: (RelayItem.CustomList) -> Unit = {},
        onSelectRelayList: (MultihopRelayListType) -> Unit = {},
        openDaitaSettings: () -> Unit = {},
        onRecentsToggleEnableClick: () -> Unit = {},
        onRefreshRelayList: () -> Unit = {},
        setMultihop: (Boolean) -> Unit = {},
    ) {

        setContentWithTheme {
            SelectLocationScreen(
                state = state,
                onSelectHop = onSelectHop,
                onModifyMultihop = onModifyMultihop,
                onSearchClick = onSearchClick,
                onBackClick = onBackClick,
                onFilterClick = onFilterClick,
                onCreateCustomList = onCreateCustomList,
                onEditCustomLists = onEditCustomLists,
                removeOwnershipFilter = removeOwnershipFilter,
                removeProviderFilter = removeProviderFilter,
                onAddLocationToList = onAddLocationToList,
                onRemoveLocationFromList = onRemoveLocationFromList,
                onEditCustomListName = onEditCustomListName,
                onEditLocationsCustomList = onEditLocationsCustomList,
                onDeleteCustomList = onDeleteCustomList,
                onSelectRelayList = onSelectRelayList,
                openDaitaSettings = openDaitaSettings,
                onRecentsToggleEnableClick = onRecentsToggleEnableClick,
                onRefreshRelayList = onRefreshRelayList,
                toggleMultihop = setMultihop,
            )
        }
    }

    @Test
    fun testShowRelayListState() =
        composeExtension.use {
            // Arrange
            every { listViewModel.uiState } returns
                MutableStateFlow(
                    Lce.Content(
                        SelectLocationListUiState(
                            relayListType = RelayListType.Single,
                            relayListItems =
                                DUMMY_RELAY_COUNTRIES.map {
                                    RelayListItem.GeoLocationItem(
                                        hop = Hop.Single(it),
                                        itemPosition = ItemPosition.Single,
                                    )
                                },
                            customLists = emptyList(),
                        )
                    )
                )
            initScreen(
                state =
                    Lc.Content(
                        SelectLocationUiState(
                            filterChips = emptyList(),
                            multihopEnabled = false,
                            relayListType = RelayListType.Single,
                            isSearchButtonEnabled = true,
                            isFilterButtonEnabled = true,
                            isRecentsEnabled = true,
                            entrySelection = null,
                            exitSelection = null,
                            tunnelErrorStateCause = null,
                            entrySelectionAllowed = true,
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
    fun customListFooterShouldShowEmptyTextWhenNoCustomList() =
        composeExtension.use {
            // Arrange
            every { listViewModel.uiState } returns
                MutableStateFlow(
                    Lce.Content(
                        SelectLocationListUiState(
                            relayListItems = listOf(RelayListItem.CustomListFooter(false)),
                            customLists = emptyList(),
                            relayListType = RelayListType.Single,
                        )
                    )
                )
            initScreen(
                state =
                    Lc.Content(
                        SelectLocationUiState(
                            filterChips = emptyList(),
                            multihopEnabled = false,
                            relayListType = RelayListType.Single,
                            isSearchButtonEnabled = true,
                            isFilterButtonEnabled = true,
                            isRecentsEnabled = true,
                            entrySelection = null,
                            exitSelection = null,
                            tunnelErrorStateCause = null,
                            entrySelectionAllowed = true,
                        )
                    )
            )

            // Assert
            onNodeWithText(CUSTOM_LISTS_EMPTY_TEXT).assertExists()
        }

    @Test
    fun whenCustomListIsClickedShouldCallOnSelectHop() =
        composeExtension.use {
            // Arrange
            val customList = Hop.Single(DUMMY_RELAY_ITEM_CUSTOM_LISTS[0])
            every { listViewModel.uiState } returns
                MutableStateFlow(
                    Lce.Content(
                        SelectLocationListUiState(
                            relayListItems = listOf(RelayListItem.CustomListItem(customList)),
                            customLists = DUMMY_RELAY_ITEM_CUSTOM_LISTS,
                            relayListType = RelayListType.Single,
                        )
                    )
                )
            val mockedOnSelectHop: (Hop) -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    Lc.Content(
                        SelectLocationUiState(
                            filterChips = emptyList(),
                            multihopEnabled = false,
                            relayListType = RelayListType.Single,
                            isSearchButtonEnabled = true,
                            isFilterButtonEnabled = true,
                            isRecentsEnabled = true,
                            entrySelection = null,
                            exitSelection = null,
                            tunnelErrorStateCause = null,
                            entrySelectionAllowed = true,
                        )
                    ),
                onSelectHop = mockedOnSelectHop,
            )

            // Act
            onNodeWithText(customList.relay.name).performClick()

            // Assert
            verify { mockedOnSelectHop(customList) }
        }

    @Test
    fun whenRecentIsClickedShouldCallOnSelectHop() =
        composeExtension.use {
            // Arrange
            val recent = Hop.Single(DUMMY_RELAY_COUNTRIES[0])
            every { listViewModel.uiState } returns
                MutableStateFlow(
                    Lce.Content(
                        SelectLocationListUiState(
                            relayListItems = listOf(RelayListItem.RecentListItem(recent)),
                            customLists = DUMMY_RELAY_ITEM_CUSTOM_LISTS,
                            relayListType = RelayListType.Single,
                        )
                    )
                )
            val mockedOnSelectHop: (Hop) -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    Lc.Content(
                        SelectLocationUiState(
                            filterChips = emptyList(),
                            multihopEnabled = false,
                            relayListType = RelayListType.Single,
                            isSearchButtonEnabled = true,
                            isFilterButtonEnabled = true,
                            isRecentsEnabled = true,
                            entrySelection = null,
                            exitSelection = null,
                            tunnelErrorStateCause = null,
                            entrySelectionAllowed = true,
                        )
                    ),
                onSelectHop = mockedOnSelectHop,
            )

            // Act
            onNodeWithText(recent.relay.name).performClick()

            // Assert
            verify { mockedOnSelectHop(recent) }
        }

    @Test
    fun whenCustomListIsLongClickedShouldShowBottomSheet() =
        composeExtension.use {
            // Arrange
            val customList = Hop.Single(DUMMY_RELAY_ITEM_CUSTOM_LISTS[0])
            every { listViewModel.uiState } returns
                MutableStateFlow(
                    Lce.Content(
                        SelectLocationListUiState(
                            relayListItems = listOf(RelayListItem.CustomListItem(hop = customList)),
                            customLists = DUMMY_RELAY_ITEM_CUSTOM_LISTS,
                            relayListType = RelayListType.Single,
                        )
                    )
                )
            val mockedOnSelectHop: (Hop) -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    Lc.Content(
                        SelectLocationUiState(
                            filterChips = emptyList(),
                            multihopEnabled = false,
                            relayListType = RelayListType.Single,
                            isSearchButtonEnabled = true,
                            isFilterButtonEnabled = true,
                            isRecentsEnabled = true,
                            entrySelection = null,
                            exitSelection = null,
                            tunnelErrorStateCause = null,
                            entrySelectionAllowed = true,
                        )
                    ),
                onSelectHop = mockedOnSelectHop,
            )

            // Act
            onNodeWithText(customList.relay.name).performLongClick()

            // Assert
            onNodeWithTag(SELECT_LOCATION_CUSTOM_LIST_BOTTOM_SHEET_TEST_TAG).assertExists()
        }

    @Test
    fun whenLocationIsLongClickedShouldShowBottomSheet() =
        composeExtension.use {
            // Arrange
            val relayItem = Hop.Single(DUMMY_RELAY_COUNTRIES[0] as RelayItem.Location)
            every { listViewModel.uiState } returns
                MutableStateFlow(
                    Lce.Content(
                        SelectLocationListUiState(
                            relayListItems =
                                listOf(
                                    RelayListItem.GeoLocationItem(
                                        relayItem,
                                        itemPosition = ItemPosition.Single,
                                    )
                                ),
                            customLists = emptyList(),
                            relayListType = RelayListType.Single,
                        )
                    )
                )
            val mockedOnSelectHop: (Hop) -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    Lc.Content(
                        SelectLocationUiState(
                            filterChips = emptyList(),
                            multihopEnabled = false,
                            relayListType = RelayListType.Single,
                            isSearchButtonEnabled = true,
                            isFilterButtonEnabled = true,
                            isRecentsEnabled = true,
                            entrySelection = null,
                            exitSelection = null,
                            tunnelErrorStateCause = null,
                            entrySelectionAllowed = true,
                        )
                    ),
                onSelectHop = mockedOnSelectHop,
            )

            // Act
            onNodeWithText(relayItem.relay.name).performLongClick()

            // Assert
            onNodeWithTag(SELECT_LOCATION_LOCATION_BOTTOM_SHEET_TEST_TAG).assertExists()
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
                            customLists = DUMMY_RELAY_ITEM_CUSTOM_LISTS,
                            relayListType = RelayListType.Single,
                        )
                    )
                )
            initScreen(
                state =
                    Lc.Content(
                        SelectLocationUiState(
                            filterChips = emptyList(),
                            multihopEnabled = false,
                            relayListType = RelayListType.Single,
                            isSearchButtonEnabled = true,
                            isFilterButtonEnabled = true,
                            isRecentsEnabled = true,
                            entrySelection = null,
                            exitSelection = null,
                            tunnelErrorStateCause = null,
                            entrySelectionAllowed = true,
                        )
                    )
            )

            // Assert
            onNodeWithTagAndText(
                    testTag = RECENT_NAME_TAG,
                    text = selectableItem.name,
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
                            customLists = DUMMY_RELAY_ITEM_CUSTOM_LISTS,
                            relayListType = RelayListType.Single,
                        )
                    )
                )
            initScreen(
                state =
                    Lc.Content(
                        SelectLocationUiState(
                            filterChips = emptyList(),
                            multihopEnabled = false,
                            relayListType = RelayListType.Single,
                            isSearchButtonEnabled = true,
                            isFilterButtonEnabled = true,
                            isRecentsEnabled = false,
                            entrySelection = null,
                            exitSelection = null,
                            tunnelErrorStateCause = null,
                            entrySelectionAllowed = true,
                        )
                    )
            )

            // Assert
            onNodeWithTagAndText(
                    testTag = GEOLOCATION_NAME_TAG,
                    text = selectableItem.name,
                    useUnmergedTree = true,
                )
                .assertExists()
        }
    }

    companion object {
        private const val CUSTOM_LISTS_EMPTY_TEXT = "To create a custom list press the \"+\""
    }
}
