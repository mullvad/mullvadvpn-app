package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.createComposeExtension
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.RelayFilterState
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.relaylist.Provider
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class FilterScreenTest {
    @JvmField @RegisterExtension val composeExtension = createComposeExtension()

    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun testDefaultState() =
        composeExtension.use {
            setContentWithTheme {
                FilterScreen(
                    uiState =
                        RelayFilterState(
                            allProviders = DUMMY_RELAY_ALL_PROVIDERS,
                            selectedOwnership = null,
                            selectedProviders = DUMMY_SELECTED_PROVIDERS,
                        ),
                    onSelectedProvider = { _, _ -> }
                )
            }
            onNodeWithText("Ownership").assertExists()
            onNodeWithText("Providers").assertExists()
        }

    @Test
    fun testIsAnyCellShowing() =
        composeExtension.use {
            setContentWithTheme {
                FilterScreen(
                    uiState =
                        RelayFilterState(
                            allProviders = DUMMY_RELAY_ALL_PROVIDERS,
                            selectedOwnership = null,
                            selectedProviders = DUMMY_SELECTED_PROVIDERS
                        ),
                    onSelectedProvider = { _, _ -> }
                )
            }
            onNodeWithText("Ownership").performClick()
            onNodeWithText("Any").assertExists()
        }

    @Test
    fun testIsMullvadCellShowing() =
        composeExtension.use {
            setContentWithTheme {
                FilterScreen(
                    uiState =
                        RelayFilterState(
                            allProviders = DUMMY_RELAY_ALL_PROVIDERS,
                            selectedOwnership = Ownership.MullvadOwned,
                            selectedProviders = DUMMY_SELECTED_PROVIDERS
                        ),
                    onSelectedProvider = { _, _ -> }
                )
            }
            onNodeWithText("Ownership").performClick()
            onNodeWithText("Mullvad owned only").assertExists()
        }

    @Test
    fun testIsRentedCellShowing() =
        composeExtension.use {
            setContentWithTheme {
                FilterScreen(
                    uiState =
                        RelayFilterState(
                            allProviders = DUMMY_RELAY_ALL_PROVIDERS,
                            selectedOwnership = Ownership.Rented,
                            selectedProviders = DUMMY_SELECTED_PROVIDERS
                        ),
                    onSelectedProvider = { _, _ -> }
                )
            }
            onNodeWithText("Ownership").performClick()
            onNodeWithText("Rented only").assertExists()
        }

    @Test
    fun testShowProviders() =
        composeExtension.use {
            setContentWithTheme {
                FilterScreen(
                    uiState =
                        RelayFilterState(
                            allProviders = DUMMY_RELAY_ALL_PROVIDERS,
                            selectedOwnership = null,
                            selectedProviders = DUMMY_SELECTED_PROVIDERS
                        ),
                    onSelectedProvider = { _, _ -> }
                )
            }

            onNodeWithText("Providers").performClick()
            onNodeWithText("Creanova").assertExists()
            onNodeWithText("100TB").assertExists()
        }

    @Test
    fun testApplyButtonClick() =
        composeExtension.use {
            val mockClickListener: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                FilterScreen(
                    uiState =
                        RelayFilterState(
                            allProviders = listOf(),
                            selectedOwnership = null,
                            selectedProviders = listOf(Provider("31173", true))
                        ),
                    onSelectedProvider = { _, _ -> },
                    onApplyClick = mockClickListener
                )
            }
            onNodeWithText("Apply").performClick()
            verify { mockClickListener() }
        }

    companion object {

        private val DUMMY_RELAY_ALL_PROVIDERS =
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

        private val DUMMY_SELECTED_PROVIDERS =
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
    }
}
