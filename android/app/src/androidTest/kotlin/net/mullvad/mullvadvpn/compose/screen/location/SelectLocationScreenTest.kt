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
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.RelayListItem
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.compose.state.SelectLocationListUiState
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_CUSTOM_LIST_BOTTOM_SHEET_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_LOCATION_BOTTOM_SHEET_TEST_TAG
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.performLongClick
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
        every { listViewModel.uiState } returns MutableStateFlow(SelectLocationListUiState.Loading)
    }

    @AfterEach
    fun teardown() {
        unmockkAll()
    }

    private fun ComposeContext.initScreen(
        state: SelectLocationUiState = SelectLocationUiState.Loading,
        onSelectRelay: (item: RelayItem) -> Unit = {},
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
        onSelectRelayList: (RelayListType) -> Unit = {},
        openDaitaSettings: () -> Unit = {},
    ) {

        setContentWithTheme {
            SelectLocationScreen(
                state = state,
                onSelectRelay = onSelectRelay,
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
            )
        }
    }

    @Test
    fun testShowRelayListState() =
        composeExtension.use {
            // Arrange
            every { listViewModel.uiState } returns
                MutableStateFlow(
                    SelectLocationListUiState.Content(
                        relayListItems =
                            DUMMY_RELAY_COUNTRIES.map { RelayListItem.GeoLocationItem(item = it) },
                        customLists = emptyList(),
                    )
                )
            initScreen(
                state =
                    SelectLocationUiState.Data(
                        // searchTerm = "",
                        filterChips = emptyList(),
                        multihopEnabled = false,
                        relayListType = RelayListType.EXIT,
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
                    SelectLocationListUiState.Content(
                        relayListItems = listOf(RelayListItem.CustomListFooter(false)),
                        customLists = emptyList(),
                    )
                )
            initScreen(
                state =
                    SelectLocationUiState.Data(
                        filterChips = emptyList(),
                        multihopEnabled = false,
                        relayListType = RelayListType.EXIT,
                    )
            )

            // Assert
            onNodeWithText(CUSTOM_LISTS_EMPTY_TEXT).assertExists()
        }

    @Test
    fun whenCustomListIsClickedShouldCallOnSelectRelay() =
        composeExtension.use {
            // Arrange
            val customList = DUMMY_RELAY_ITEM_CUSTOM_LISTS[0]
            every { listViewModel.uiState } returns
                MutableStateFlow(
                    SelectLocationListUiState.Content(
                        relayListItems = listOf(RelayListItem.CustomListItem(customList)),
                        customLists = DUMMY_RELAY_ITEM_CUSTOM_LISTS,
                    )
                )
            val mockedOnSelectRelay: (RelayItem) -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    SelectLocationUiState.Data(
                        filterChips = emptyList(),
                        multihopEnabled = false,
                        relayListType = RelayListType.EXIT,
                    ),
                onSelectRelay = mockedOnSelectRelay,
            )

            // Act
            onNodeWithText(customList.name).performClick()

            // Assert
            verify { mockedOnSelectRelay(customList) }
        }

    @Test
    fun whenCustomListIsLongClickedShouldShowBottomSheet() =
        composeExtension.use {
            // Arrange
            val customList = DUMMY_RELAY_ITEM_CUSTOM_LISTS[0]
            every { listViewModel.uiState } returns
                MutableStateFlow(
                    SelectLocationListUiState.Content(
                        relayListItems = listOf(RelayListItem.CustomListItem(item = customList)),
                        customLists = DUMMY_RELAY_ITEM_CUSTOM_LISTS,
                    )
                )
            val mockedOnSelectRelay: (RelayItem) -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    SelectLocationUiState.Data(
                        // searchTerm = "",
                        filterChips = emptyList(),
                        multihopEnabled = false,
                        relayListType = RelayListType.EXIT,
                    ),
                onSelectRelay = mockedOnSelectRelay,
            )

            // Act
            onNodeWithText(customList.name).performLongClick()

            // Assert
            onNodeWithTag(SELECT_LOCATION_CUSTOM_LIST_BOTTOM_SHEET_TEST_TAG)
        }

    @Test
    fun whenLocationIsLongClickedShouldShowBottomSheet() =
        composeExtension.use {
            // Arrange
            val relayItem = DUMMY_RELAY_COUNTRIES[0]
            every { listViewModel.uiState } returns
                MutableStateFlow(
                    SelectLocationListUiState.Content(
                        relayListItems = listOf(RelayListItem.GeoLocationItem(relayItem)),
                        customLists = emptyList(),
                    )
                )
            val mockedOnSelectRelay: (RelayItem) -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    SelectLocationUiState.Data(
                        filterChips = emptyList(),
                        multihopEnabled = false,
                        relayListType = RelayListType.EXIT,
                    ),
                onSelectRelay = mockedOnSelectRelay,
            )

            // Act
            onNodeWithText(relayItem.name).performLongClick()

            // Assert
            onNodeWithTag(SELECT_LOCATION_LOCATION_BOTTOM_SHEET_TEST_TAG)
        }

    companion object {
        private const val CUSTOM_LISTS_EMPTY_TEXT = "To create a custom list press the \"︙\""
    }
}
