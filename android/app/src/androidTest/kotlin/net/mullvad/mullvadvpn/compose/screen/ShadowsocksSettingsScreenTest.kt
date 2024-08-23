package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import io.mockk.coVerify
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.ShadowsocksSettingsState
import net.mullvad.mullvadvpn.compose.test.SHADOWSOCKS_CUSTOM_PORT_TEXT_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SHADOWSOCKS_PORT_ITEM_X_TEST_TAG
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.onNodeWithTagAndText
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class ShadowsocksSettingsScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @Test
    fun testSelectTcpOverUdpPortOption() =
        composeExtension.use {
            // Arrange
            val onObfuscationPortSelected: (Constraint<Port>) -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                ShadowsocksSettingsScreen(
                    state = ShadowsocksSettingsState(port = Constraint.Any),
                    onObfuscationPortSelected = onObfuscationPortSelected
                )
            }

            // Act
            onNodeWithTagAndText(
                    testTag = String.format(SHADOWSOCKS_PORT_ITEM_X_TEST_TAG, 443),
                    text = "443"
                )
                .assertExists()
                .performClick()

            // Assert
            coVerify(exactly = 1) { onObfuscationPortSelected.invoke(Constraint.Only(Port(443))) }
        }

    @Test
    fun testShowShadowsocksCustomPort() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                ShadowsocksSettingsScreen(
                    state = ShadowsocksSettingsState(customPort = Port(4000)),
                )
            }

            // Assert
            onNodeWithText("4000").assertExists()
        }

    @Test
    fun testSelectShadowsocksCustomPort() =
        composeExtension.use {
            // Arrange
            val onObfuscationPortSelected: (Constraint<Port>) -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                ShadowsocksSettingsScreen(
                    state =
                        ShadowsocksSettingsState(
                            port = Constraint.Only(Port(4000)),
                            customPort = Port(4000)
                        ),
                    onObfuscationPortSelected = onObfuscationPortSelected
                )
            }

            // Act
            onNodeWithTag(testTag = SHADOWSOCKS_CUSTOM_PORT_TEXT_TEST_TAG).performClick()

            // Assert
            verify { onObfuscationPortSelected.invoke(Constraint.Only(Port(4000))) }
        }
}
