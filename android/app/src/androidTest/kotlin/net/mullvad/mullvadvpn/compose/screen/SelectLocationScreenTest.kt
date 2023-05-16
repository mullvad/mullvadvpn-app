package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import io.mockk.MockKAnnotations
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.model.RelayEndpointData
import net.mullvad.mullvadvpn.model.RelayListCity
import net.mullvad.mullvadvpn.model.RelayListCountry
import net.mullvad.mullvadvpn.model.WireguardRelayEndpointData
import net.mullvad.mullvadvpn.relaylist.Relay
import net.mullvad.mullvadvpn.relaylist.RelayList
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
        composeTestRule.setContent { SelectLocationScreen(uiState = SelectLocationUiState.Loading) }

        // Assert
        composeTestRule.apply {
            onNodeWithText(
                    "While connected, your real location is masked with a private and secure location in the selected region."
                )
                .assertExists()
            onNodeWithText("Relay Country 1").assertDoesNotExist()
            onNodeWithText("Relay City 1").assertDoesNotExist()
            onNodeWithText("Relay 1").assertDoesNotExist()
            onNodeWithText("Relay Country 2").assertDoesNotExist()
        }
    }

    @Test
    fun testShowRelayListState() {
        // Arrange
        composeTestRule.setContent {
            SelectLocationScreen(
                uiState =
                    SelectLocationUiState.Data.Show(
                        countries = DUMMY_RELAY_LIST.countries,
                        selectedRelay = null
                    )
            )
        }

        // Assert
        composeTestRule.apply {
            onNodeWithText(
                    "While connected, your real location is masked with a private and secure location in the selected region."
                )
                .assertExists()
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
                        SelectLocationUiState.Data.Show(
                            countries =
                                DUMMY_RELAY_LIST.countries.apply {
                                    this[0].expanded = true
                                    this[0].cities[0].expanded = true
                                },
                            selectedRelay = DUMMY_RELAY_LIST.countries[0].cities[0].relays[0]
                        )
                )
            }
        }

        // Assert
        composeTestRule.apply {
            onNodeWithText(
                    "While connected, your real location is masked with a private and secure location in the selected region."
                )
                .assertExists()
            onNodeWithText("Relay Country 1").assertExists()
            onNodeWithText("Relay City 1").assertExists()
            onNodeWithText("Relay host 1").assertExists()
            onNodeWithText("Relay Country 2").assertExists()
            onNodeWithText("Relay City 2").assertDoesNotExist()
            onNodeWithText("Relay host 2").assertDoesNotExist()
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

        private val DUMMY_RELAY_LIST =
            RelayList(
                net.mullvad.mullvadvpn.model.RelayList(
                    arrayListOf(DUMMY_RELAY_COUNTRY_1, DUMMY_RELAY_COUNTRY_2)
                )
            )
    }
}
