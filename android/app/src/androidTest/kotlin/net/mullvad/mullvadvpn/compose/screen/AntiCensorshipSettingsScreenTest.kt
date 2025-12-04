package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertTextContains
import androidx.compose.ui.test.hasAnyAncestor
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollToNode
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.AntiCensorshipSettingsUiState
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.ui.tag.BUTTON_ARROW_RIGHT_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_ANTI_CENSORSHIP_SETTINGS_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_OBFUSCATION_WG_PORT_TEST_TAG
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.toLc
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class AntiCensorshipSettingsScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    private fun createDefaultUiState(
        obfuscationMode: ObfuscationMode = ObfuscationMode.Auto,
        selectedUdp2TcpObfuscationPort: Constraint<Port> = Constraint.Any,
        selectedShadowsocksObfuscationPort: Constraint<Port> = Constraint.Any,
        selectedWireguardPort: Constraint<Port> = Constraint.Any,
        isModal: Boolean = false,
    ) =
        AntiCensorshipSettingsUiState.from(
            isModal = isModal,
            obfuscationMode = obfuscationMode,
            selectedUdp2TcpObfuscationPort = selectedUdp2TcpObfuscationPort,
            selectedShadowsocksObfuscationPort = selectedShadowsocksObfuscationPort,
            selectedWireguardPort = selectedWireguardPort,
        )

    private fun ComposeContext.initScreen(
        state: Lc<Unit, AntiCensorshipSettingsUiState> = createDefaultUiState().toLc(),
        onBackClick: () -> Unit = {},
        onSelectObfuscationMode: (obfuscationMode: ObfuscationMode) -> Unit = {},
        navigateToShadowSocksSettings: () -> Unit = {},
        navigateToUdp2TcpSettings: () -> Unit = {},
        navigateToWireguardPortSettings: () -> Unit = {},
    ) {
        setContentWithTheme {
            AntiCensorshipSettingsScreen(
                state = state,
                navigateToShadowSocksSettings = navigateToShadowSocksSettings,
                navigateToUdp2TcpSettings = navigateToUdp2TcpSettings,
                navigateToWireguardPortSettings = navigateToWireguardPortSettings,
                onBackClick = onBackClick,
                onSelectObfuscationMode = onSelectObfuscationMode,
            )
        }
    }

    @Test
    fun testDefaultState() =
        composeExtension.use {
            // Arrange
            initScreen()

            // Assert
            onNodeWithText("WireGuard port").assertExists()
            onNodeWithText("LWO").assertExists()
            onNodeWithText("QUIC").assertExists()
            onNodeWithText("WireGuard port").assertExists()
            onNodeWithText("Shadowsocks").assertExists()
            onNodeWithText("UDP-over-TCP").assertExists()
            onNodeWithText("Automatic").assertExists()
            onNodeWithText("Off").assertExists()
        }

    @Test
    fun testWireguardPortShouldBeDisplayed() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    createDefaultUiState(selectedWireguardPort = Constraint.Only(Port(53))).toLc()
            )

            // Act
            onNodeWithTag(LAZY_LIST_ANTI_CENSORSHIP_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(WIREGUARD_OBFUSCATION_WG_PORT_TEST_TAG))

            // Assert
            onNodeWithTag(WIREGUARD_OBFUSCATION_WG_PORT_TEST_TAG).assertTextContains("Port: 53")
        }

    @Test
    fun testSelectWireguardPortShouldCallOnSelectObfuscationMode() =
        composeExtension.use {
            // Arrange
            val mockSelectObfuscationMode: (ObfuscationMode) -> Unit = mockk(relaxed = true)

            initScreen(
                state =
                    createDefaultUiState(selectedWireguardPort = Constraint.Only(Port(53))).toLc(),
                onSelectObfuscationMode = mockSelectObfuscationMode,
            )

            onNodeWithTag(LAZY_LIST_ANTI_CENSORSHIP_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(WIREGUARD_OBFUSCATION_WG_PORT_TEST_TAG))

            // Act
            onNodeWithTag(WIREGUARD_OBFUSCATION_WG_PORT_TEST_TAG).performClick()

            // Assert
            verify(exactly = 1) { mockSelectObfuscationMode.invoke(ObfuscationMode.WireguardPort) }
        }

    @Test
    fun testShowUdp2TcpCustomPortDialog() =
        composeExtension.use {
            val mockedClickHandler: () -> Unit = mockk(relaxed = true)

            // Arrange
            initScreen(
                createDefaultUiState(selectedUdp2TcpObfuscationPort = Constraint.Only(Port(53)))
                    .toLc(),
                navigateToUdp2TcpSettings = mockedClickHandler,
            )

            onNodeWithTag(LAZY_LIST_ANTI_CENSORSHIP_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL_TEST_TAG))

            onNode(
                    hasTestTag(BUTTON_ARROW_RIGHT_TEST_TAG) and
                        hasAnyAncestor(
                            hasTestTag(WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL_TEST_TAG)
                        ),
                    useUnmergedTree = true,
                )
                .assertExists()
                .performClick()

            // Assert
            verify(exactly = 1) { mockedClickHandler.invoke() }
        }
}
