package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.RelayFilterUiState
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.lib.model.ProviderId
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class FilterScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    fun setup() {
        MockKAnnotations.init(this)
    }

    private fun ComposeContext.initScreen(
        state: RelayFilterUiState = RelayFilterUiState(),
        onBackClick: () -> Unit = {},
        onApplyClick: () -> Unit = {},
        onSelectedOwnership: (ownership: Ownership?) -> Unit = {},
        onAllProviderCheckChange: (isChecked: Boolean) -> Unit = {},
        onSelectedProvider: (checked: Boolean, provider: Provider) -> Unit = { _, _ -> },
    ) {
        setContentWithTheme {
            FilterScreen(
                state = state,
                onBackClick = onBackClick,
                onApplyClick = onApplyClick,
                onSelectedOwnership = onSelectedOwnership,
                onAllProviderCheckChange = onAllProviderCheckChange,
                onSelectedProvider = onSelectedProvider,
            )
        }
    }

    @Test
    fun testDefaultState() =
        composeExtension.use {
            initScreen(
                state =
                    RelayFilterUiState(
                        allProviders = DUMMY_RELAY_ALL_PROVIDERS,
                        selectedOwnership = null,
                        selectedProviders = DUMMY_SELECTED_PROVIDERS,
                    )
            )
            onNodeWithText("Ownership").assertExists()
            onNodeWithText("Providers").assertExists()
        }

    @Test
    fun testIsAnyCellShowing() =
        composeExtension.use {
            initScreen(
                state =
                    RelayFilterUiState(
                        allProviders = DUMMY_RELAY_ALL_PROVIDERS,
                        selectedOwnership = null,
                        selectedProviders = DUMMY_SELECTED_PROVIDERS,
                    )
            )
            onNodeWithText("Ownership").performClick()
            onNodeWithText("Any").assertExists()
        }

    @Test
    fun testIsMullvadCellShowing() =
        composeExtension.use {
            initScreen(
                state =
                    RelayFilterUiState(
                        allProviders = DUMMY_RELAY_ALL_PROVIDERS,
                        selectedOwnership = Ownership.MullvadOwned,
                        selectedProviders = DUMMY_SELECTED_PROVIDERS,
                    )
            )
            onNodeWithText("Ownership").performClick()
            onNodeWithText("Mullvad owned only").assertExists()
        }

    @Test
    fun testIsRentedCellShowing() =
        composeExtension.use {
            initScreen(
                state =
                    RelayFilterUiState(
                        allProviders = DUMMY_RELAY_ALL_PROVIDERS,
                        selectedOwnership = Ownership.Rented,
                        selectedProviders = DUMMY_SELECTED_PROVIDERS,
                    )
            )
            onNodeWithText("Ownership").performClick()
            onNodeWithText("Rented only").assertExists()
        }

    @Test
    fun testShowProviders() =
        composeExtension.use {
            initScreen(
                state =
                    RelayFilterUiState(
                        allProviders = DUMMY_RELAY_ALL_PROVIDERS,
                        selectedOwnership = null,
                        selectedProviders = DUMMY_SELECTED_PROVIDERS,
                    )
            )

            onNodeWithText("Providers").performClick()
            onNodeWithText("Creanova").assertExists()
            onNodeWithText("100TB").assertExists()
        }

    @Test
    fun testApplyButtonClick() =
        composeExtension.use {
            val mockClickListener: () -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    RelayFilterUiState(
                        allProviders = listOf(),
                        selectedOwnership = null,
                        selectedProviders =
                            listOf(Provider(ProviderId("31173"), Ownership.MullvadOwned)),
                    ),
                onApplyClick = mockClickListener,
            )
            onNodeWithText("Apply").performClick()
            verify { mockClickListener() }
        }

    companion object {
        private val DUMMY_RELAY_ALL_PROVIDERS =
            listOf(
                Provider(ProviderId("31173"), Ownership.MullvadOwned),
                Provider(ProviderId("100TB"), Ownership.Rented),
                Provider(ProviderId("Blix"), Ownership.MullvadOwned),
                Provider(ProviderId("Creanova"), Ownership.MullvadOwned),
                Provider(ProviderId("DataPacket"), Ownership.Rented),
                Provider(ProviderId("HostRoyale"), Ownership.Rented),
                Provider(ProviderId("hostuniversal"), Ownership.Rented),
                Provider(ProviderId("iRegister"), Ownership.Rented),
                Provider(ProviderId("M247"), Ownership.Rented),
                Provider(ProviderId("Makonix"), Ownership.Rented),
                Provider(ProviderId("PrivateLayer"), Ownership.Rented),
                Provider(ProviderId("ptisp"), Ownership.Rented),
                Provider(ProviderId("Qnax"), Ownership.Rented),
                Provider(ProviderId("Quadranet"), Ownership.Rented),
                Provider(ProviderId("techfutures"), Ownership.Rented),
                Provider(ProviderId("Tzulo"), Ownership.Rented),
                Provider(ProviderId("xtom"), Ownership.Rented),
            )

        private val DUMMY_SELECTED_PROVIDERS =
            listOf(
                Provider(ProviderId("31173"), Ownership.MullvadOwned),
                Provider(ProviderId("100TB"), Ownership.Rented),
                Provider(ProviderId("Blix"), Ownership.MullvadOwned),
                Provider(ProviderId("Creanova"), Ownership.MullvadOwned),
                Provider(ProviderId("DataPacket"), Ownership.Rented),
                Provider(ProviderId("HostRoyale"), Ownership.Rented),
                Provider(ProviderId("hostuniversal"), Ownership.Rented),
                Provider(ProviderId("iRegister"), Ownership.Rented),
                Provider(ProviderId("M247"), Ownership.Rented),
                Provider(ProviderId("Makonix"), Ownership.Rented),
                Provider(ProviderId("PrivateLayer"), Ownership.Rented),
                Provider(ProviderId("ptisp"), Ownership.Rented),
                Provider(ProviderId("Qnax"), Ownership.Rented),
                Provider(ProviderId("Quadranet"), Ownership.Rented),
                Provider(ProviderId("techfutures"), Ownership.Rented),
                Provider(ProviderId("Tzulo"), Ownership.Rented),
                Provider(ProviderId("xtom"), Ownership.Rented),
            )
    }
}
