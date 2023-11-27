package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.MutableSharedFlow
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.RelayFilterState
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.relaylist.Provider
import org.junit.Rule
import org.junit.Test

class FilterScreenTest {
    @get:Rule val composeTestRule = createComposeRule()

    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun testDefaultState() {
        composeTestRule.setContentWithTheme {
            FilterScreen(
                uiState =
                    RelayFilterState(
                        allProviders = DUMMY_RELAY_ALL_PROVIDERS,
                        selectedOwnership = null,
                        selectedProviders = DUMMY_SELECTED_PROVIDERS,
                    ),
                uiCloseAction = MutableSharedFlow(),
                onSelectedProviders = { _, _ -> }
            )
        }
        composeTestRule.apply {
            onNodeWithText("Ownership").assertExists()
            onNodeWithText("Providers").assertExists()
        }
    }

    @Test
    fun testIsAnyCellShowing() {
        composeTestRule.setContentWithTheme {
            FilterScreen(
                uiState =
                    RelayFilterState(
                        allProviders = DUMMY_RELAY_ALL_PROVIDERS,
                        selectedOwnership = null,
                        selectedProviders = DUMMY_SELECTED_PROVIDERS
                    ),
                uiCloseAction = MutableSharedFlow(),
                onSelectedProviders = { _, _ -> }
            )
        }
        composeTestRule.apply {
            onNodeWithText("Ownership").performClick()
            onNodeWithText("Any").assertExists()
        }
    }

    @Test
    fun testIsMullvadCellShowing() {
        composeTestRule.setContentWithTheme {
            FilterScreen(
                uiState =
                    RelayFilterState(
                        allProviders = DUMMY_RELAY_ALL_PROVIDERS,
                        selectedOwnership = Ownership.MullvadOwned,
                        selectedProviders = DUMMY_SELECTED_PROVIDERS
                    ),
                uiCloseAction = MutableSharedFlow(),
                onSelectedProviders = { _, _ -> }
            )
        }
        composeTestRule.apply {
            onNodeWithText("Ownership").performClick()
            onNodeWithText("Mullvad owned only").performClick()
        }
    }

    @Test
    fun testIsRentedCellShowing() {
        composeTestRule.setContentWithTheme {
            FilterScreen(
                uiState =
                    RelayFilterState(
                        allProviders = DUMMY_RELAY_ALL_PROVIDERS,
                        selectedOwnership = Ownership.Rented,
                        selectedProviders = DUMMY_SELECTED_PROVIDERS
                    ),
                uiCloseAction = MutableSharedFlow(),
                onSelectedProviders = { _, _ -> }
            )
        }
        composeTestRule.apply {
            onNodeWithText("Ownership").performClick()
            onNodeWithText("Rented only").assertExists()
        }
    }

    @Test
    fun testShowProviders() {
        composeTestRule.setContentWithTheme {
            FilterScreen(
                uiState =
                    RelayFilterState(
                        allProviders = DUMMY_RELAY_ALL_PROVIDERS,
                        selectedOwnership = null,
                        selectedProviders = DUMMY_SELECTED_PROVIDERS
                    ),
                uiCloseAction = MutableSharedFlow(),
                onSelectedProviders = { _, _ -> }
            )
        }

        composeTestRule.apply {
            onNodeWithText("Providers").performClick()
            onNodeWithText("Creanova").assertExists()
            onNodeWithText("Creanova").assertExists()
            onNodeWithText("100TB").assertExists()
        }
    }

    @Test
    fun testApplyButtonClick() {
        val mockClickListener: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            FilterScreen(
                uiState =
                    RelayFilterState(
                        allProviders = listOf(),
                        selectedOwnership = null,
                        selectedProviders = listOf(Provider("31173", true))
                    ),
                uiCloseAction = MutableSharedFlow(),
                onSelectedProviders = { _, _ -> },
                onApplyClick = mockClickListener
            )
        }
        composeTestRule.onNodeWithText("Apply").performClick()
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
