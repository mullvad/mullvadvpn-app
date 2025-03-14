package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import io.mockk.verify
import java.time.Duration
import java.time.Instant
import java.time.ZonedDateTime
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.ConnectUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.test.CONNECT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.CONNECT_CARD_HEADER_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.RECONNECT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.TOP_BAR_ACCOUNT_BUTTON
import net.mullvad.mullvadvpn.lib.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.GeoIpLocation
import net.mullvad.mullvadvpn.lib.model.InAppNotification
import net.mullvad.mullvadvpn.lib.model.TransportProtocol
import net.mullvad.mullvadvpn.lib.model.TunnelEndpoint
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.model.VersionInfo
import net.mullvad.mullvadvpn.lib.shared.compose.test.NOTIFICATION_BANNER_ACTION
import net.mullvad.mullvadvpn.lib.shared.compose.test.NOTIFICATION_BANNER_TEXT_ACTION
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@Suppress("LargeClass")
class ConnectScreenTest {
    @OptIn(ExperimentalTestApi::class)
    @JvmField
    @RegisterExtension
    val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    @AfterEach
    fun teardown() {
        unmockkAll()
    }

    @Suppress("LongParameterList")
    private fun ComposeContext.initScreen(
        state: ConnectUiState = ConnectUiState.INITIAL,
        onDisconnectClick: () -> Unit = {},
        onReconnectClick: () -> Unit = {},
        onConnectClick: () -> Unit = {},
        onCancelClick: () -> Unit = {},
        onSwitchLocationClick: () -> Unit = {},
        onOpenAppListing: () -> Unit = {},
        onManageAccountClick: () -> Unit = {},
        onSettingsClick: () -> Unit = {},
        onAccountClick: () -> Unit = {},
        onDismissNewDeviceClick: () -> Unit = {},
        onChangelogClick: () -> Unit = {},
        onDismissChangelogClick: () -> Unit = {},
    ) {
        setContentWithTheme {
            ConnectScreen(
                state = state,
                onDisconnectClick = onDisconnectClick,
                onReconnectClick = onReconnectClick,
                onConnectClick = onConnectClick,
                onCancelClick = onCancelClick,
                onSwitchLocationClick = onSwitchLocationClick,
                onOpenAppListing = onOpenAppListing,
                onManageAccountClick = onManageAccountClick,
                onSettingsClick = onSettingsClick,
                onAccountClick = onAccountClick,
                onDismissNewDeviceClick = onDismissNewDeviceClick,
                onChangelogClick = onChangelogClick,
                onDismissChangelogClick = onDismissChangelogClick,
            )
        }
    }

    @Test
    fun testDefaultState() {
        composeExtension.use {
            // Arrange
            initScreen()

            // Assert
            onNodeWithText("DISCONNECTED").assertExists()
            onNodeWithText("Connect").assertExists()
        }
    }

    @Test
    fun testConnectingState() {
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    ConnectUiState(
                        location = null,
                        selectedRelayItemTitle = null,
                        tunnelState = TunnelState.Connecting(null, null, emptyList()),
                        showLocation = false,
                        deviceName = "",
                        daysLeftUntilExpiry = null,
                        inAppNotification = InAppNotification.TunnelStateBlocked,
                        isPlayBuild = false,
                    )
            )

            // Assert
            onNodeWithTag(CIRCULAR_PROGRESS_INDICATOR).assertExists()
            onNodeWithText("CONNECTING...").assertExists()
            onNodeWithText("Switch location").assertExists()
            onNodeWithText("Cancel").assertExists()
            onNodeWithText("BLOCKING INTERNET").assertExists()
        }
    }

    @Test
    fun testConnectedState() {
        composeExtension.use {
            // Arrange
            val mockTunnelEndpoint: TunnelEndpoint = mockk(relaxed = true)
            initScreen(
                state =
                    ConnectUiState(
                        location = null,
                        selectedRelayItemTitle = null,
                        tunnelState = TunnelState.Connected(mockTunnelEndpoint, null, emptyList()),
                        showLocation = false,
                        deviceName = "",
                        daysLeftUntilExpiry = null,
                        inAppNotification = null,
                        isPlayBuild = false,
                    )
            )

            // Assert
            onNodeWithText("CONNECTED").assertExists()
            onNodeWithText("Switch location").assertExists()
            onNodeWithText("Disconnect").assertExists()
        }
    }

    @Test
    fun testDisconnectingState() {
        composeExtension.use {
            // Arrange
            val mockLocationName = "Home"
            initScreen(
                state =
                    ConnectUiState(
                        location = null,
                        selectedRelayItemTitle = mockLocationName,
                        tunnelState = TunnelState.Disconnecting(ActionAfterDisconnect.Nothing),
                        showLocation = true,
                        deviceName = "",
                        daysLeftUntilExpiry = null,
                        inAppNotification = null,
                        isPlayBuild = false,
                    )
            )

            // Assert
            onNodeWithText("DISCONNECTING...").assertExists()
            onNodeWithText(mockLocationName).assertExists()
            onNodeWithText("Disconnect").assertExists()
        }
    }

    @Test
    fun testDisconnectedState() {
        composeExtension.use {
            // Arrange
            val mockLocationName = "Home"
            initScreen(
                state =
                    ConnectUiState(
                        location = null,
                        selectedRelayItemTitle = mockLocationName,
                        tunnelState = TunnelState.Disconnected(),
                        showLocation = true,
                        deviceName = "",
                        daysLeftUntilExpiry = null,
                        inAppNotification = null,
                        isPlayBuild = false,
                    )
            )

            // Assert
            onNodeWithText("DISCONNECTED").assertExists()
            onNodeWithText(mockLocationName).assertExists()
            onNodeWithText("Connect").assertExists()
        }
    }

    @Test
    fun testErrorStateBlocked() {
        composeExtension.use {
            // Arrange
            val mockLocationName = "Home"
            initScreen(
                state =
                    ConnectUiState(
                        location = null,
                        selectedRelayItemTitle = mockLocationName,
                        tunnelState =
                            TunnelState.Error(ErrorState(ErrorStateCause.StartTunnelError, true)),
                        showLocation = true,
                        deviceName = "",
                        daysLeftUntilExpiry = null,
                        inAppNotification =
                            InAppNotification.TunnelStateError(
                                ErrorState(ErrorStateCause.StartTunnelError, true)
                            ),
                        isPlayBuild = false,
                    )
            )

            // Assert
            onNodeWithText("BLOCKED CONNECTION").assertExists()
            onNodeWithText(mockLocationName).assertExists()
            onNodeWithText("Disconnect").assertExists()
            onNodeWithText("BLOCKING INTERNET").assertExists()
        }
    }

    @Test
    fun testErrorStateNotBlocked() {
        composeExtension.use {
            // Arrange
            val mockLocationName = "Home"
            initScreen(
                state =
                    ConnectUiState(
                        location = null,
                        selectedRelayItemTitle = mockLocationName,
                        tunnelState =
                            TunnelState.Error(ErrorState(ErrorStateCause.StartTunnelError, false)),
                        showLocation = true,
                        deviceName = "",
                        daysLeftUntilExpiry = null,
                        inAppNotification =
                            InAppNotification.TunnelStateError(
                                ErrorState(ErrorStateCause.StartTunnelError, false)
                            ),
                        isPlayBuild = false,
                    )
            )

            // Assert
            onNodeWithText("FAILED TO CONNECT").assertExists()
            onNodeWithText(mockLocationName).assertExists()
            onNodeWithText("Dismiss").assertExists()
            onNodeWithText(text = "Critical error (your attention is required)", ignoreCase = true)
                .assertExists()
        }
    }

    @Test
    fun testReconnectingState() {
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    ConnectUiState(
                        location = null,
                        selectedRelayItemTitle = null,
                        tunnelState = TunnelState.Disconnecting(ActionAfterDisconnect.Reconnect),
                        showLocation = false,
                        deviceName = "",
                        daysLeftUntilExpiry = null,
                        inAppNotification = InAppNotification.TunnelStateBlocked,
                        isPlayBuild = false,
                    )
            )

            // Assert
            onNodeWithTag(CIRCULAR_PROGRESS_INDICATOR).assertExists()
            onNodeWithText("CONNECTING...").assertExists()
            onNodeWithText("Switch location").assertExists()
            onNodeWithText("Disconnect").assertExists()
            onNodeWithText("BLOCKING INTERNET").assertExists()
        }
    }

    @Test
    fun testDisconnectingBlockState() {
        composeExtension.use {
            // Arrange
            val mockLocationName = "Home"
            initScreen(
                state =
                    ConnectUiState(
                        location = null,
                        selectedRelayItemTitle = mockLocationName,
                        tunnelState = TunnelState.Disconnecting(ActionAfterDisconnect.Block),
                        showLocation = true,
                        deviceName = "",
                        daysLeftUntilExpiry = null,
                        inAppNotification = InAppNotification.TunnelStateBlocked,
                        isPlayBuild = false,
                    )
            )

            // Assert
            onNodeWithText("BLOCKING...").assertExists()
            onNodeWithText(mockLocationName).assertExists()
            onNodeWithText("Disconnect").assertExists()
            onNodeWithText("BLOCKING INTERNET").assertExists()
        }
    }

    @Test
    fun testClickSelectLocationButton() {
        composeExtension.use {
            // Arrange
            val mockLocationName = "Home"
            val mockedClickHandler: () -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    ConnectUiState(
                        location = null,
                        selectedRelayItemTitle = mockLocationName,
                        tunnelState = TunnelState.Disconnected(),
                        showLocation = false,
                        deviceName = "",
                        daysLeftUntilExpiry = null,
                        inAppNotification = null,
                        isPlayBuild = false,
                    ),
                onSwitchLocationClick = mockedClickHandler,
            )

            // Act
            onNodeWithTag(SELECT_LOCATION_BUTTON_TEST_TAG).performClick()

            // Assert
            verify { mockedClickHandler.invoke() }
        }
    }

    @Test
    fun testOnDisconnectClick() {
        composeExtension.use {
            // Arrange
            val mockTunnelEndpoint: TunnelEndpoint = mockk(relaxed = true)
            val mockedClickHandler: () -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    ConnectUiState(
                        location = null,
                        selectedRelayItemTitle = null,
                        tunnelState = TunnelState.Connected(mockTunnelEndpoint, null, emptyList()),
                        showLocation = false,
                        deviceName = "",
                        daysLeftUntilExpiry = null,
                        inAppNotification = null,
                        isPlayBuild = false,
                    ),
                onDisconnectClick = mockedClickHandler,
            )

            // Act
            onNodeWithTag(CONNECT_BUTTON_TEST_TAG).performClick()

            // Assert
            verify { mockedClickHandler.invoke() }
        }
    }

    @Test
    fun testOnReconnectClick() {
        composeExtension.use {
            // Arrange
            val mockTunnelEndpoint: TunnelEndpoint = mockk(relaxed = true)
            val mockedClickHandler: () -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    ConnectUiState(
                        location = null,
                        selectedRelayItemTitle = null,
                        tunnelState = TunnelState.Connected(mockTunnelEndpoint, null, emptyList()),
                        showLocation = false,
                        deviceName = "",
                        daysLeftUntilExpiry = null,
                        inAppNotification = null,
                        isPlayBuild = false,
                    ),
                onReconnectClick = mockedClickHandler,
            )

            // Act
            onNodeWithTag(RECONNECT_BUTTON_TEST_TAG).performClick()

            // Assert
            verify { mockedClickHandler.invoke() }
        }
    }

    @Test
    fun testOnConnectClick() {
        composeExtension.use {
            // Arrange
            val mockedClickHandler: () -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    ConnectUiState(
                        location = null,
                        selectedRelayItemTitle = null,
                        tunnelState = TunnelState.Disconnected(),
                        showLocation = false,
                        deviceName = "",
                        daysLeftUntilExpiry = null,
                        inAppNotification = null,
                        isPlayBuild = false,
                    ),
                onConnectClick = mockedClickHandler,
            )

            // Act
            onNodeWithTag(CONNECT_BUTTON_TEST_TAG).performClick()

            // Assert
            verify { mockedClickHandler.invoke() }
        }
    }

    @Test
    fun testOnCancelClick() {
        composeExtension.use {
            // Arrange
            val mockedClickHandler: () -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    ConnectUiState(
                        location = null,
                        selectedRelayItemTitle = null,
                        tunnelState = TunnelState.Connecting(null, null, emptyList()),
                        showLocation = false,
                        deviceName = "",
                        daysLeftUntilExpiry = null,
                        inAppNotification = null,
                        isPlayBuild = false,
                    ),
                onCancelClick = mockedClickHandler,
            )

            // Act
            onNodeWithTag(CONNECT_BUTTON_TEST_TAG).performClick()

            // Assert
            verify { mockedClickHandler.invoke() }
        }
    }

    @Test
    fun showConnectionDetails() {
        composeExtension.use {
            // Arrange
            val mockLocation: GeoIpLocation = mockk(relaxed = true)
            val mockTunnelEndpoint: TunnelEndpoint = mockk(relaxed = true)
            val mockHostName = "Host-Name"
            val inHost = "Host"
            val inPort = 99
            val inProtocol = TransportProtocol.Udp
            every { mockLocation.hostname } returns mockHostName
            every { mockLocation.entryHostname } returns null

            // In
            every { mockTunnelEndpoint.obfuscation } returns null
            every { mockTunnelEndpoint.endpoint.address.address.hostAddress } returns inHost
            every { mockTunnelEndpoint.endpoint.address.port } returns inPort
            every { mockTunnelEndpoint.endpoint.protocol } returns inProtocol

            // Out Ipv4
            val outIpv4 = "ipv4address"
            every { mockLocation.ipv4?.hostAddress } returns outIpv4

            // Out Ipv6
            val outIpv6 = "ipv6address"
            every { mockLocation.ipv6?.hostAddress } returns outIpv6

            initScreen(
                state =
                    ConnectUiState(
                        location = mockLocation,
                        selectedRelayItemTitle = null,
                        tunnelState =
                            TunnelState.Connected(mockTunnelEndpoint, mockLocation, emptyList()),
                        showLocation = false,
                        deviceName = "",
                        daysLeftUntilExpiry = null,
                        inAppNotification = null,
                        isPlayBuild = false,
                    )
            )

            // Act
            onNodeWithTag(CONNECT_CARD_HEADER_TEST_TAG).performClick()

            // Assert
            onNodeWithText(mockHostName).assertExists()
            onNodeWithText("In").assertExists()
            onNodeWithText("$inHost:$inPort UDP").assertExists()

            onNodeWithText("Out IPv4").assertExists()
            onNodeWithText(outIpv4).assertExists()

            onNodeWithText("Out IPv6").assertExists()
            onNodeWithText(outIpv6).assertExists()
        }
    }

    @Test
    fun testUnsupportedVersionNotification() {
        composeExtension.use {
            // Arrange
            val versionInfo = VersionInfo(currentVersion = "1.0", isSupported = false)
            initScreen(
                state =
                    ConnectUiState(
                        location = null,
                        selectedRelayItemTitle = null,
                        tunnelState = TunnelState.Connecting(null, null, emptyList()),
                        showLocation = false,
                        deviceName = "",
                        daysLeftUntilExpiry = null,
                        inAppNotification = InAppNotification.UnsupportedVersion(versionInfo),
                        isPlayBuild = false,
                    )
            )

            // Assert
            onNodeWithText("UNSUPPORTED VERSION").assertExists()
            onNodeWithText(
                    "Your privacy might be at risk with this unsupported app version. Please update now."
                )
                .assertExists()
        }
    }

    @Test
    fun testAccountExpiredNotification() {
        composeExtension.use {
            // Arrange
            val expiryDate = ZonedDateTime.parse("2020-11-11T10:10Z")
            initScreen(
                state =
                    ConnectUiState(
                        location = null,
                        selectedRelayItemTitle = null,
                        tunnelState = TunnelState.Connecting(null, null, emptyList()),
                        showLocation = false,
                        deviceName = "",
                        daysLeftUntilExpiry = null,
                        inAppNotification =
                            InAppNotification.AccountExpiry(
                                Duration.between(Instant.now(), expiryDate)
                            ),
                        isPlayBuild = false,
                    )
            )

            // Assert
            onNodeWithText("ACCOUNT CREDIT EXPIRES SOON").assertExists()
            onNodeWithText("Out of time").assertExists()
        }
    }

    @Test
    fun testOnUpdateVersionClick() {
        composeExtension.use {
            // Arrange
            val mockedClickHandler: () -> Unit = mockk(relaxed = true)
            val versionInfo = VersionInfo(isSupported = false, currentVersion = "")
            initScreen(
                onOpenAppListing = mockedClickHandler,
                state =
                    ConnectUiState(
                        location = null,
                        selectedRelayItemTitle = null,
                        tunnelState = TunnelState.Connecting(null, null, emptyList()),
                        showLocation = false,
                        deviceName = "",
                        daysLeftUntilExpiry = null,
                        inAppNotification = InAppNotification.UnsupportedVersion(versionInfo),
                        isPlayBuild = false,
                    ),
            )

            // Act
            onNodeWithTag(NOTIFICATION_BANNER_ACTION).performClick()

            // Assert
            verify { mockedClickHandler.invoke() }
        }
    }

    @Test
    fun testOnShowAccountClick() {
        composeExtension.use {
            // Arrange
            val mockedClickHandler: () -> Unit = mockk(relaxed = true)
            val expiryDate = ZonedDateTime.parse("2020-11-11T10:10Z")
            initScreen(
                onManageAccountClick = mockedClickHandler,
                state =
                    ConnectUiState(
                        location = null,
                        selectedRelayItemTitle = null,
                        tunnelState = TunnelState.Connecting(null, null, emptyList()),
                        showLocation = false,
                        deviceName = "",
                        daysLeftUntilExpiry = null,
                        inAppNotification =
                            InAppNotification.AccountExpiry(
                                Duration.between(Instant.now(), expiryDate)
                            ),
                        isPlayBuild = false,
                    ),
            )

            // Act
            onNodeWithTag(NOTIFICATION_BANNER_ACTION).performClick()

            // Assert
            verify { mockedClickHandler.invoke() }
        }
    }

    @Test
    fun testOnNewChangelogMessageClick() {
        composeExtension.use {
            // Arrange
            val mockedClickHandler: () -> Unit = mockk(relaxed = true)
            initScreen(
                onChangelogClick = mockedClickHandler,
                state =
                    ConnectUiState(
                        location = null,
                        selectedRelayItemTitle = null,
                        tunnelState = TunnelState.Connecting(null, null, emptyList()),
                        showLocation = false,
                        deviceName = "",
                        daysLeftUntilExpiry = null,
                        inAppNotification = InAppNotification.NewVersionChangelog,
                        isPlayBuild = false,
                    ),
            )

            // Act
            onNodeWithTag(NOTIFICATION_BANNER_TEXT_ACTION).performClick()

            // Assert
            verify { mockedClickHandler.invoke() }
        }
    }

    @Test
    fun testOpenAccountView() {
        composeExtension.use {
            // Arrange
            val onAccountClickMockk: () -> Unit = mockk(relaxed = true)
            initScreen(state = ConnectUiState.INITIAL, onAccountClick = onAccountClickMockk)

            // Assert
            onNodeWithTag(TOP_BAR_ACCOUNT_BUTTON).performClick()

            verify(exactly = 1) { onAccountClickMockk() }
        }
    }
}
