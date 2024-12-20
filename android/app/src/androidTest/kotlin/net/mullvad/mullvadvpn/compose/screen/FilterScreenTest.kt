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
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
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
        onSelectedOwnership: (ownership: Constraint<Ownership>) -> Unit = {},
        onAllProviderCheckChange: (isChecked: Boolean) -> Unit = {},
        onSelectedProvider: (checked: Boolean, provider: ProviderId) -> Unit = { _, _ -> },
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
                        providerToOwnerships = DUMMY_RELAY_ALL_PROVIDERS,
                        selectedOwnership = Constraint.Any,
                        selectedProviders = Constraint.Only(DUMMY_SELECTED_PROVIDERS),
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
                        providerToOwnerships = DUMMY_RELAY_ALL_PROVIDERS,
                        selectedOwnership = Constraint.Any,
                        selectedProviders = Constraint.Only(DUMMY_SELECTED_PROVIDERS),
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
                        providerToOwnerships = DUMMY_RELAY_ALL_PROVIDERS,
                        selectedOwnership = Constraint.Only(Ownership.MullvadOwned),
                        selectedProviders = Constraint.Only(DUMMY_SELECTED_PROVIDERS),
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
                        providerToOwnerships = DUMMY_RELAY_ALL_PROVIDERS,
                        selectedOwnership = Constraint.Only(Ownership.Rented),
                        selectedProviders = Constraint.Only(DUMMY_SELECTED_PROVIDERS),
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
                        providerToOwnerships = DUMMY_RELAY_ALL_PROVIDERS,
                        selectedOwnership = Constraint.Any,
                        selectedProviders = Constraint.Only(DUMMY_SELECTED_PROVIDERS),
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
                        providerToOwnerships = DUMMY_RELAY_ALL_PROVIDERS,
                        selectedOwnership = Constraint.Any,
                        selectedProviders = Constraint.Only(listOf(ProviderId("31173"))),
                    ),
                onApplyClick = mockClickListener,
            )
            onNodeWithText("Apply").performClick()
            verify { mockClickListener() }
        }

    @Test
    fun ensureSelectedProviderIsShowEvenThoughItIsNotInAllProviders() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    RelayFilterUiState(
                        providerToOwnerships = DUMMY_RELAY_ALL_PROVIDERS,
                        selectedOwnership = null,
                        selectedProviders = listOf(ProviderId("1RemovedProvider")),
                    )
            )

            // Act
            onNodeWithText("Providers").performClick()
            // Asset
            onNodeWithText("1RemovedProvider (removed)").assertExists()
        }

    companion object {
        private val DUMMY_RELAY_ALL_PROVIDERS =
            mapOf(
                ProviderId("31173") to setOf(Ownership.MullvadOwned),
                ProviderId("100TB") to setOf(Ownership.Rented),
                ProviderId("Blix") to setOf(Ownership.MullvadOwned),
                ProviderId("Creanova") to setOf(Ownership.MullvadOwned),
                ProviderId("DataPacket") to setOf(Ownership.Rented),
                ProviderId("HostRoyale") to setOf(Ownership.Rented),
                ProviderId("hostuniversal") to setOf(Ownership.Rented),
                ProviderId("iRegister") to setOf(Ownership.Rented),
                ProviderId("M247") to setOf(Ownership.Rented),
                ProviderId("Makonix") to setOf(Ownership.Rented),
                ProviderId("PrivateLayer") to setOf(Ownership.Rented),
                ProviderId("ptisp") to setOf(Ownership.Rented),
                ProviderId("Qnax") to setOf(Ownership.Rented),
                ProviderId("Quadranet") to setOf(Ownership.Rented),
                ProviderId("techfutures") to setOf(Ownership.Rented),
                ProviderId("Tzulo") to setOf(Ownership.Rented),
                ProviderId("xtom") to setOf(Ownership.Rented),
            )

        private val DUMMY_SELECTED_PROVIDERS = DUMMY_RELAY_ALL_PROVIDERS.keys.toList().dropLast(3)
    }
}
