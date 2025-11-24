package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.ShadowsocksSettingsUiState
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.ui.tag.SHADOWSOCKS_CUSTOM_PORT_TEXT_TEST_TAG
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.toLc
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class ShadowsocksSettingsScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    private fun ComposeContext.initScreen(
        state: Lc<Unit, ShadowsocksSettingsUiState>,
        navigateToCustomPortDialog: (port: Port?) -> Unit = {},
        onObfuscationPortSelected: (Constraint<Port>) -> Unit = {},
        onBackClick: () -> Unit = {},
    ) {
        setContentWithTheme {
            ShadowsocksSettingsScreen(
                state = state,
                navigateToCustomPortDialog = navigateToCustomPortDialog,
                onObfuscationPortSelected = onObfuscationPortSelected,
                onBackClick = onBackClick,
            )
        }
    }

    @Test
    fun testShowShadowsocksCustomPort() =
        composeExtension.use {
            // Arrange
            initScreen(state = ShadowsocksSettingsUiState(customPort = Port(4000)).toLc())

            // Assert
            onNodeWithText("Port: 4000").assertExists()
        }

    @Test
    fun testSelectShadowsocksCustomPort() =
        composeExtension.use {
            // Arrange
            val onObfuscationPortSelected: (Constraint<Port>) -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    ShadowsocksSettingsUiState(
                            port = Constraint.Only(Port(4000)),
                            customPort = Port(4000),
                        )
                        .toLc(),
                onObfuscationPortSelected = onObfuscationPortSelected,
            )

            // Act
            onNodeWithTag(testTag = SHADOWSOCKS_CUSTOM_PORT_TEXT_TEST_TAG).performClick()

            // Assert
            verify { onObfuscationPortSelected.invoke(Constraint.Only(Port(4000))) }
        }
}
