package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.RelayFilterState
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Provider
import net.mullvad.mullvadvpn.model.ProviderId
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class FilterScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun testDefaultState() =
        composeExtension.use {
            setContentWithTheme {
                FilterScreen(
                    state =
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
                    state =
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
                    state =
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
                    state =
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
                    state =
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
                    state =
                        RelayFilterState(
                            allProviders = listOf(),
                            selectedOwnership = null,
                            selectedProviders =
                                listOf(Provider(ProviderId("31173"), Ownership.MullvadOwned))
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
                Provider(ProviderId("xtom"), Ownership.Rented)
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
                Provider(ProviderId("xtom"), Ownership.Rented)
            )
    }
}
