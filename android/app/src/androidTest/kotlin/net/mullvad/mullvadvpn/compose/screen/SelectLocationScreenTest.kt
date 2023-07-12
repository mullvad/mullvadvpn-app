package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performTextInput
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.MutableSharedFlow
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.model.PortRange
import net.mullvad.mullvadvpn.model.RelayEndpointData
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.RelayListCity
import net.mullvad.mullvadvpn.model.RelayListCountry
import net.mullvad.mullvadvpn.model.WireguardEndpointData
import net.mullvad.mullvadvpn.model.WireguardRelayEndpointData
import net.mullvad.mullvadvpn.relaylist.toRelayCountries
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class SelectLocationScreenTest {
    @get:Rule val composeTestRule = createComposeRule()

    @Before
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun testDefaultState() {
        // Arrange
        composeTestRule.setContent {
            SelectLocationScreen(
                uiState = SelectLocationUiState.Loading,
                uiCloseAction = MutableSharedFlow()
            )
        }

        // Assert
        composeTestRule.apply { onNodeWithTag(CIRCULAR_PROGRESS_INDICATOR).assertExists() }
    }

    @Test
    fun testShowRelayListState() {
        // Arrange
        composeTestRule.setContent {
            SelectLocationScreen(
                uiState =
                    SelectLocationUiState.ShowData(
                        countries = DUMMY_RELAY_COUNTRIES,
                        selectedRelay = null
                    ),
                uiCloseAction = MutableSharedFlow()
            )
        }

        // Assert
        composeTestRule.apply {
            onNodeWithText("Relay Country 1").assertExists()
            onNodeWithText("Relay City 1").assertDoesNotExist()
            onNodeWithText("Relay host 1").assertDoesNotExist()
            onNodeWithText("Relay Country 2").assertExists()
            onNodeWithText("Relay City 2").assertDoesNotExist()
            onNodeWithText("Relay host 2").assertDoesNotExist()
        }
    }

    @Test
    fun testShowRelayListStateSelected() {
        // Arrange
        composeTestRule.setContent {
            AppTheme {
                SelectLocationScreen(
                    uiState =
                        SelectLocationUiState.ShowData(
                            countries =
                                DUMMY_RELAY_COUNTRIES.apply {
                                    this[0].expanded = true
                                    this[0].cities[0].expanded = true
                                },
                            selectedRelay = DUMMY_RELAY_COUNTRIES[0].cities[0].relays[0]
                        ),
                    uiCloseAction = MutableSharedFlow()
                )
            }
        }

        // Assert
        composeTestRule.apply {
            onNodeWithText("Relay Country 1").assertExists()
            onNodeWithText("Relay City 1").assertExists()
            onNodeWithText("Relay host 1").assertExists()
            onNodeWithText("Relay Country 2").assertExists()
            onNodeWithText("Relay City 2").assertDoesNotExist()
            onNodeWithText("Relay host 2").assertDoesNotExist()
        }
    }

    @Test
    fun testSearchInput() {
        // Arrange
        val mockedSearchTermInput: (String) -> Unit = mockk(relaxed = true)
        composeTestRule.setContent {
            AppTheme {
                SelectLocationScreen(
                    uiState =
                        SelectLocationUiState.ShowData(
                            countries = emptyList(),
                            selectedRelay = null
                        ),
                    uiCloseAction = MutableSharedFlow(),
                    onSearchTermInput = mockedSearchTermInput
                )
            }
        }
        val mockSearchString = "SEARCH"

        // Act
        composeTestRule.apply { onNodeWithText("Search for...").performTextInput(mockSearchString) }

        // Assert
        verify { mockedSearchTermInput.invoke(mockSearchString) }
    }

    @Test
    fun testSearchTermNotFound() {
        // Arrange
        val mockedSearchTermInput: (String) -> Unit = mockk(relaxed = true)
        val mockSearchString = "SEARCH"
        composeTestRule.setContent {
            AppTheme {
                SelectLocationScreen(
                    uiState =
                        SelectLocationUiState.NoSearchResultFound(searchTerm = mockSearchString),
                    uiCloseAction = MutableSharedFlow(),
                    onSearchTermInput = mockedSearchTermInput
                )
            }
        }

        // Assert
        composeTestRule.apply {
            onNodeWithText("No result for $mockSearchString.", substring = true).assertExists()
            onNodeWithText("Try a different search", substring = true).assertExists()
        }
    }

    companion object {
        private val DUMMY_RELAY_1 =
            net.mullvad.mullvadvpn.model.Relay(
                "Relay host 1",
                true,
                RelayEndpointData.Wireguard(WireguardRelayEndpointData)
            )
        private val DUMMY_RELAY_2 =
            net.mullvad.mullvadvpn.model.Relay(
                "Relay host 2",
                true,
                RelayEndpointData.Wireguard(WireguardRelayEndpointData)
            )
        private val DUMMY_RELAY_CITY_1 =
            RelayListCity("Relay City 1", "RCi1", arrayListOf(DUMMY_RELAY_1))
        private val DUMMY_RELAY_CITY_2 =
            RelayListCity("Relay City 2", "RCi2", arrayListOf(DUMMY_RELAY_2))
        private val DUMMY_RELAY_COUNTRY_1 =
            RelayListCountry("Relay Country 1", "RCo1", arrayListOf(DUMMY_RELAY_CITY_1))
        private val DUMMY_RELAY_COUNTRY_2 =
            RelayListCountry("Relay Country 2", "RCo2", arrayListOf(DUMMY_RELAY_CITY_2))

        private val DUMMY_WIREGUARD_PORT_RANGES = ArrayList<PortRange>()
        private val DUMMY_WIREGUARD_ENDPOINT_DATA =
            WireguardEndpointData(DUMMY_WIREGUARD_PORT_RANGES)

        private val DUMMY_RELAY_COUNTRIES =
            RelayList(
                    arrayListOf(DUMMY_RELAY_COUNTRY_1, DUMMY_RELAY_COUNTRY_2),
                    DUMMY_WIREGUARD_ENDPOINT_DATA
                )
                .toRelayCountries()
    }
}
