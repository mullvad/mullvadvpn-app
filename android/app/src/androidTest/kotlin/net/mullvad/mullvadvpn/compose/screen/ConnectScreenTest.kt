package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.ConnectUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.test.CONNECT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LOCATION_INFO_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.NOTIFICATION_BANNER_ACTION
import net.mullvad.mullvadvpn.compose.test.RECONNECT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SCROLLABLE_COLUMN_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.TOP_BAR_ACCOUNT_BUTTON
import net.mullvad.mullvadvpn.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.model.ErrorState
import net.mullvad.mullvadvpn.model.ErrorStateCause
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TransportProtocol
import net.mullvad.mullvadvpn.model.TunnelEndpoint
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.ui.VersionInfo
import org.joda.time.DateTime
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

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

    @Test
    fun testDefaultState() {
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                ConnectScreen(
                    state = ConnectUiState.INITIAL,
                )
            }

            // Assert
            onNodeWithTag(SCROLLABLE_COLUMN_TEST_TAG).assertExists()
            onNodeWithText("UNSECURED CONNECTION").assertExists()
            onNodeWithText("Secure my connection").assertExists()
        }
    }

    @Test
    fun testConnectingState() {
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                ConnectScreen(
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = null,
                            tunnelState = TunnelState.Connecting(null, null),
                            inAddress = null,
                            outAddress = "",
                            showLocation = false,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification = InAppNotification.TunnelStateBlocked,
                            isPlayBuild = false
                        ),
                )
            }

            // Assert
            onNodeWithTag(CIRCULAR_PROGRESS_INDICATOR).assertExists()
            onNodeWithText("CREATING SECURE CONNECTION").assertExists()
            onNodeWithText("Switch location").assertExists()
            onNodeWithText("Cancel").assertExists()
            onNodeWithText("BLOCKING INTERNET").assertExists()
        }
    }

    @Test
    fun testConnectingStateQuantumSecured() {
        composeExtension.use {
            // Arrange
            val mockTunnelEndpoint: TunnelEndpoint = mockk(relaxed = true)
            every { mockTunnelEndpoint.quantumResistant } returns true
            setContentWithTheme {
                ConnectScreen(
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = null,
                            tunnelState =
                                TunnelState.Connecting(endpoint = mockTunnelEndpoint, null),
                            inAddress = null,
                            outAddress = "",
                            showLocation = false,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification = InAppNotification.TunnelStateBlocked,
                            isPlayBuild = false
                        ),
                )
            }

            // Assert
            onNodeWithTag(CIRCULAR_PROGRESS_INDICATOR).assertExists()
            onNodeWithText("CREATING QUANTUM SECURE CONNECTION").assertExists()
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
            setContentWithTheme {
                ConnectScreen(
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = null,
                            tunnelState = TunnelState.Connected(mockTunnelEndpoint, null),
                            inAddress = null,
                            outAddress = "",
                            showLocation = false,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification = null,
                            isPlayBuild = false
                        ),
                )
            }

            // Assert
            onNodeWithText("SECURE CONNECTION").assertExists()
            onNodeWithText("Switch location").assertExists()
            onNodeWithText("Disconnect").assertExists()
        }
    }

    @Test
    fun testConnectedStateQuantumSecured() {
        composeExtension.use {
            // Arrange
            val mockTunnelEndpoint: TunnelEndpoint = mockk(relaxed = true)
            every { mockTunnelEndpoint.quantumResistant } returns true
            setContentWithTheme {
                ConnectScreen(
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = null,
                            tunnelState = TunnelState.Connected(mockTunnelEndpoint, null),
                            inAddress = null,
                            outAddress = "",
                            showLocation = false,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification = null,
                            isPlayBuild = false
                        ),
                )
            }

            // Assert
            onNodeWithText("QUANTUM SECURE CONNECTION").assertExists()
            onNodeWithText("Switch location").assertExists()
            onNodeWithText("Disconnect").assertExists()
        }
    }

    @Test
    fun testDisconnectingState() {
        composeExtension.use {
            // Arrange
            val mockLocationName = "Home"
            setContentWithTheme {
                ConnectScreen(
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = mockLocationName,
                            tunnelState = TunnelState.Disconnecting(ActionAfterDisconnect.Nothing),
                            inAddress = null,
                            outAddress = "",
                            showLocation = true,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification = null,
                            isPlayBuild = false
                        ),
                )
            }

            // Assert
            onNodeWithText("UNSECURED CONNECTION").assertExists()
            onNodeWithText(mockLocationName).assertExists()
            onNodeWithText("Disconnect").assertExists()
        }
    }

    @Test
    fun testDisconnectedState() {
        composeExtension.use {
            // Arrange
            val mockLocationName = "Home"
            setContentWithTheme {
                ConnectScreen(
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = mockLocationName,
                            tunnelState = TunnelState.Disconnected(),
                            inAddress = null,
                            outAddress = "",
                            showLocation = true,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification = null,
                            isPlayBuild = false
                        ),
                )
            }

            // Assert
            onNodeWithText("UNSECURED CONNECTION").assertExists()
            onNodeWithText(mockLocationName).assertExists()
            onNodeWithText("Secure my connection").assertExists()
        }
    }

    @Test
    fun testErrorStateBlocked() {
        composeExtension.use {
            // Arrange
            val mockLocationName = "Home"
            setContentWithTheme {
                ConnectScreen(
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = mockLocationName,
                            tunnelState =
                                TunnelState.Error(
                                    ErrorState(ErrorStateCause.StartTunnelError, true)
                                ),
                            inAddress = null,
                            outAddress = "",
                            showLocation = true,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification =
                                InAppNotification.TunnelStateError(
                                    ErrorState(ErrorStateCause.StartTunnelError, true)
                                ),
                            isPlayBuild = false
                        ),
                )
            }

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
            setContentWithTheme {
                ConnectScreen(
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = mockLocationName,
                            tunnelState =
                                TunnelState.Error(
                                    ErrorState(ErrorStateCause.StartTunnelError, false)
                                ),
                            inAddress = null,
                            outAddress = "",
                            showLocation = true,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification =
                                InAppNotification.TunnelStateError(
                                    ErrorState(ErrorStateCause.StartTunnelError, false)
                                ),
                            isPlayBuild = false
                        ),
                )
            }

            // Assert
            onNodeWithText("FAILED TO SECURE CONNECTION").assertExists()
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
            setContentWithTheme {
                ConnectScreen(
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = null,
                            tunnelState =
                                TunnelState.Disconnecting(ActionAfterDisconnect.Reconnect),
                            inAddress = null,
                            outAddress = "",
                            showLocation = false,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification = InAppNotification.TunnelStateBlocked,
                            isPlayBuild = false
                        ),
                )
            }

            // Assert
            onNodeWithTag(CIRCULAR_PROGRESS_INDICATOR).assertExists()
            onNodeWithText("CREATING SECURE CONNECTION").assertExists()
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
            setContentWithTheme {
                ConnectScreen(
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = mockLocationName,
                            tunnelState = TunnelState.Disconnecting(ActionAfterDisconnect.Block),
                            inAddress = null,
                            outAddress = "",
                            showLocation = true,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification = InAppNotification.TunnelStateBlocked,
                            isPlayBuild = false
                        ),
                )
            }

            // Assert
            onNodeWithText("SECURE CONNECTION").assertExists()
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
            setContentWithTheme {
                ConnectScreen(
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = mockLocationName,
                            tunnelState = TunnelState.Disconnected(),
                            inAddress = null,
                            outAddress = "",
                            showLocation = false,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification = null,
                            isPlayBuild = false
                        ),
                    onSwitchLocationClick = mockedClickHandler
                )
            }

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
            setContentWithTheme {
                ConnectScreen(
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = null,
                            tunnelState = TunnelState.Connected(mockTunnelEndpoint, null),
                            inAddress = null,
                            outAddress = "",
                            showLocation = false,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification = null,
                            isPlayBuild = false
                        ),
                    onDisconnectClick = mockedClickHandler
                )
            }

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
            setContentWithTheme {
                ConnectScreen(
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = null,
                            tunnelState = TunnelState.Connected(mockTunnelEndpoint, null),
                            inAddress = null,
                            outAddress = "",
                            showLocation = false,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification = null,
                            isPlayBuild = false
                        ),
                    onReconnectClick = mockedClickHandler
                )
            }

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
            setContentWithTheme {
                ConnectScreen(
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = null,
                            tunnelState = TunnelState.Disconnected(),
                            inAddress = null,
                            outAddress = "",
                            showLocation = false,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification = null,
                            isPlayBuild = false
                        ),
                    onConnectClick = mockedClickHandler
                )
            }

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
            setContentWithTheme {
                ConnectScreen(
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = null,
                            tunnelState = TunnelState.Connecting(null, null),
                            inAddress = null,
                            outAddress = "",
                            showLocation = false,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification = null,
                            isPlayBuild = false
                        ),
                    onCancelClick = mockedClickHandler
                )
            }

            // Act
            onNodeWithTag(CONNECT_BUTTON_TEST_TAG).performClick()

            // Assert
            verify { mockedClickHandler.invoke() }
        }
    }

    @Test
    fun showLocationInfo() {
        composeExtension.use {
            // Arrange
            val mockLocation: GeoIpLocation = mockk(relaxed = true)
            val mockTunnelEndpoint: TunnelEndpoint = mockk(relaxed = true)
            val mockHostName = "Host-Name"
            val mockPort = 99
            val mockHost = "Host"
            val mockProtocol = TransportProtocol.Udp
            val mockInAddress = Triple(mockHost, mockPort, mockProtocol)
            val mockOutAddress = "HostAddressV4 / HostAddressV4"
            every { mockLocation.hostname } returns mockHostName
            setContentWithTheme {
                ConnectScreen(
                    state =
                        ConnectUiState(
                            location = mockLocation,
                            selectedRelayItemTitle = null,
                            tunnelState = TunnelState.Connected(mockTunnelEndpoint, null),
                            inAddress = mockInAddress,
                            outAddress = mockOutAddress,
                            showLocation = false,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification = null,
                            isPlayBuild = false
                        ),
                )
            }

            // Act
            onNodeWithTag(LOCATION_INFO_TEST_TAG).performClick()

            // Assert
            onNodeWithText(mockHostName).assertExists()
            onNodeWithText("WireGuard").assertExists()
            onNodeWithText("In $mockHost:$mockPort UDP").assertExists()
            onNodeWithText("Out $mockOutAddress").assertExists()
        }
    }

    @Test
    fun testOutdatedVersionNotification() {
        composeExtension.use {
            // Arrange
            val versionInfo =
                VersionInfo(
                    currentVersion = "1.0",
                    upgradeVersion = "1.1",
                    isOutdated = true,
                    isSupported = true
                )
            setContentWithTheme {
                ConnectScreen(
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = null,
                            tunnelState = TunnelState.Connecting(null, null),
                            inAddress = null,
                            outAddress = "",
                            showLocation = false,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification = InAppNotification.UpdateAvailable(versionInfo),
                            isPlayBuild = false
                        ),
                )
            }

            // Assert
            onNodeWithText("UPDATE AVAILABLE").assertExists()
            onNodeWithText("Install Mullvad VPN (1.1) to stay up to date").assertExists()
        }
    }

    @Test
    fun testUnsupportedVersionNotification() {
        composeExtension.use {
            // Arrange
            val versionInfo =
                VersionInfo(
                    currentVersion = "1.0",
                    upgradeVersion = "1.1",
                    isOutdated = true,
                    isSupported = false
                )
            setContentWithTheme {
                ConnectScreen(
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = null,
                            tunnelState = TunnelState.Connecting(null, null),
                            inAddress = null,
                            outAddress = "",
                            showLocation = false,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification = InAppNotification.UnsupportedVersion(versionInfo),
                            isPlayBuild = false
                        ),
                )
            }

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
            val expiryDate = DateTime(2020, 11, 11, 10, 10)
            setContentWithTheme {
                ConnectScreen(
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = null,
                            tunnelState = TunnelState.Connecting(null, null),
                            inAddress = null,
                            outAddress = "",
                            showLocation = false,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification = InAppNotification.AccountExpiry(expiryDate),
                            isPlayBuild = false
                        ),
                )
            }

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
            val versionInfo =
                VersionInfo(
                    currentVersion = "1.0",
                    upgradeVersion = "1.1",
                    isOutdated = true,
                    isSupported = false
                )
            setContentWithTheme {
                ConnectScreen(
                    onUpdateVersionClick = mockedClickHandler,
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = null,
                            tunnelState = TunnelState.Connecting(null, null),
                            inAddress = null,
                            outAddress = "",
                            showLocation = false,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification = InAppNotification.UnsupportedVersion(versionInfo),
                            isPlayBuild = false
                        ),
                )
            }

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
            val expiryDate = DateTime(2020, 11, 11, 10, 10)
            setContentWithTheme {
                ConnectScreen(
                    onManageAccountClick = mockedClickHandler,
                    state =
                        ConnectUiState(
                            location = null,
                            selectedRelayItemTitle = null,
                            tunnelState = TunnelState.Connecting(null, null),
                            inAddress = null,
                            outAddress = "",
                            showLocation = false,
                            deviceName = "",
                            daysLeftUntilExpiry = null,
                            inAppNotification = InAppNotification.AccountExpiry(expiryDate),
                            isPlayBuild = false
                        ),
                )
            }

            // Act
            onNodeWithTag(NOTIFICATION_BANNER_ACTION).performClick()

            // Assert
            verify { mockedClickHandler.invoke() }
        }
    }

    @Test
    fun testOpenAccountView() {
        composeExtension.use {
            // Arrange
            val onAccountClickMockk: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                ConnectScreen(state = ConnectUiState.INITIAL, onAccountClick = onAccountClickMockk)
            }

            // Assert
            onNodeWithTag(TOP_BAR_ACCOUNT_BUTTON).performClick()

            verify(exactly = 1) { onAccountClickMockk() }
        }
    }
}
