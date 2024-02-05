package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performTextInput
import de.mannodermaus.junit5.compose.createComposeExtension
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.RelayListState
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.PortRange
import net.mullvad.mullvadvpn.model.RelayEndpointData
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.RelayListCity
import net.mullvad.mullvadvpn.model.RelayListCountry
import net.mullvad.mullvadvpn.model.WireguardEndpointData
import net.mullvad.mullvadvpn.model.WireguardRelayEndpointData
import net.mullvad.mullvadvpn.relaylist.toRelayCountries
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class SelectLocationScreenTest {
    @JvmField @RegisterExtension val composeExtension = createComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun testDefaultState() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                SelectLocationScreen(
                    uiState = SelectLocationUiState.Loading,
                )
            }

            // Assert
            onNodeWithTag(CIRCULAR_PROGRESS_INDICATOR).assertExists()
        }

    @Test
    fun testShowRelayListState() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                SelectLocationScreen(
                    uiState =
                        SelectLocationUiState.Data(
                            relayListState =
                                RelayListState.RelayList(
                                    countries = DUMMY_RELAY_COUNTRIES,
                                    selectedItem = null
                                ),
                            selectedOwnership = null,
                            selectedProvidersCount = 0,
                            searchTerm = ""
                        ),
                )
            }

            // Assert
            onNodeWithText("Relay Country 1").assertExists()
            onNodeWithText("Relay City 1").assertDoesNotExist()
            onNodeWithText("Relay host 1").assertDoesNotExist()
            onNodeWithText("Relay Country 2").assertExists()
            onNodeWithText("Relay City 2").assertDoesNotExist()
            onNodeWithText("Relay host 2").assertDoesNotExist()
        }

    @Test
    fun testShowRelayListStateSelected() =
        composeExtension.use {
            val updatedDummyList =
                DUMMY_RELAY_COUNTRIES.let {
                    val cities = it[0].cities.toMutableList()
                    val city = cities.removeAt(0)
                    cities.add(0, city.copy(expanded = true))

                    val mutableRelayList = it.toMutableList()
                    mutableRelayList[0] = it[0].copy(expanded = true, cities = cities.toList())
                    mutableRelayList
                }

            // Arrange
            setContentWithTheme {
                SelectLocationScreen(
                    uiState =
                        SelectLocationUiState.Data(
                            relayListState =
                                RelayListState.RelayList(
                                    countries = updatedDummyList,
                                    selectedItem = updatedDummyList[0].cities[0].relays[0]
                                ),
                            selectedOwnership = null,
                            selectedProvidersCount = 0,
                            searchTerm = ""
                        ),
                )
            }

            // Assert
            onNodeWithText("Relay Country 1").assertExists()
            onNodeWithText("Relay City 1").assertExists()
            onNodeWithText("Relay host 1").assertExists()
            onNodeWithText("Relay Country 2").assertExists()
            onNodeWithText("Relay City 2").assertDoesNotExist()
            onNodeWithText("Relay host 2").assertDoesNotExist()
        }

    @Test
    fun testSearchInput() =
        composeExtension.use {
            // Arrange
            val mockedSearchTermInput: (String) -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                SelectLocationScreen(
                    uiState =
                        SelectLocationUiState.Data(
                            relayListState =
                                RelayListState.RelayList(
                                    countries = emptyList(),
                                    selectedItem = null
                                ),
                            selectedOwnership = null,
                            selectedProvidersCount = 0,
                            searchTerm = ""
                        ),
                    onSearchTermInput = mockedSearchTermInput
                )
            }
            val mockSearchString = "SEARCH"

            // Act
            onNodeWithText("Search for...").performTextInput(mockSearchString)

            // Assert
            verify { mockedSearchTermInput.invoke(mockSearchString) }
        }

    @Test
    fun testSearchTermNotFound() =
        composeExtension.use {
            // Arrange
            val mockedSearchTermInput: (String) -> Unit = mockk(relaxed = true)
            val mockSearchString = "SEARCH"
            setContentWithTheme {
                SelectLocationScreen(
                    uiState =
                        SelectLocationUiState.Data(
                            relayListState = RelayListState.Empty,
                            selectedOwnership = null,
                            selectedProvidersCount = 0,
                            searchTerm = mockSearchString
                        ),
                    onSearchTermInput = mockedSearchTermInput
                )
            }

            // Assert
            onNodeWithText("No result for $mockSearchString.", substring = true).assertExists()
            onNodeWithText("Try a different search", substring = true).assertExists()
        }

    companion object {
        private val DUMMY_RELAY_1 =
            net.mullvad.mullvadvpn.model.Relay(
                hostname = "Relay host 1",
                active = true,
                endpointData = RelayEndpointData.Wireguard(WireguardRelayEndpointData),
                owned = true,
                provider = "PROVIDER"
            )
        private val DUMMY_RELAY_2 =
            net.mullvad.mullvadvpn.model.Relay(
                hostname = "Relay host 2",
                active = true,
                endpointData = RelayEndpointData.Wireguard(WireguardRelayEndpointData),
                owned = true,
                provider = "PROVIDER"
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
                    DUMMY_WIREGUARD_ENDPOINT_DATA,
                )
                .toRelayCountries(ownership = Constraint.Any(), providers = Constraint.Any())
    }
}
