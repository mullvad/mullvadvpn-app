package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.onNodeWithContentDescription
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
import net.mullvad.mullvadvpn.compose.state.VpnSettingsUiState
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_LAST_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_QUANTUM_ITEM_OFF_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_VPN_SETTINGS_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_WIREGUARD_CUSTOM_PORT_NUMBER_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_WIREGUARD_OBFUSCATION_TITLE_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.onNodeWithTagAndText
import net.mullvad.mullvadvpn.viewmodel.CustomDnsItem
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class VpnSettingsScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    private fun ComposeContext.initScreen(
        state: VpnSettingsUiState = VpnSettingsUiState.createDefault(),
        navigateToContentBlockersInfo: () -> Unit = {},
        navigateToAutoConnectScreen: () -> Unit = {},
        navigateToCustomDnsInfo: () -> Unit = {},
        navigateToMalwareInfo: () -> Unit = {},
        navigateToObfuscationInfo: () -> Unit = {},
        navigateToQuantumResistanceInfo: () -> Unit = {},
        navigateToWireguardPortInfo: (availablePortRanges: List<PortRange>) -> Unit = {},
        navigateToLocalNetworkSharingInfo: () -> Unit = {},
        navigateToWireguardPortDialog: () -> Unit = {},
        navigateToServerIpOverrides: () -> Unit = {},
        onToggleBlockTrackers: (Boolean) -> Unit = {},
        onToggleBlockAds: (Boolean) -> Unit = {},
        onToggleBlockMalware: (Boolean) -> Unit = {},
        onToggleLocalNetworkSharing: (Boolean) -> Unit = {},
        onToggleBlockAdultContent: (Boolean) -> Unit = {},
        onToggleBlockGambling: (Boolean) -> Unit = {},
        onToggleBlockSocialMedia: (Boolean) -> Unit = {},
        navigateToMtuDialog: (mtu: Mtu?) -> Unit = {},
        navigateToDns: (index: Int?, address: String?) -> Unit = { _, _ -> },
        onToggleDnsClick: (Boolean) -> Unit = {},
        onBackClick: () -> Unit = {},
        onSelectObfuscationMode: (obfuscationMode: ObfuscationMode) -> Unit = {},
        onSelectQuantumResistanceSetting: (quantumResistant: QuantumResistantState) -> Unit = {},
        onWireguardPortSelected: (port: Constraint<Port>) -> Unit = {},
        navigateToShadowSocksSettings: () -> Unit = {},
        navigateToUdp2TcpSettings: () -> Unit = {},
        onToggleAutoStartAndConnectOnBoot: (Boolean) -> Unit = {},
        onSelectDeviceIpVersion: (Constraint<IpVersion>) -> Unit = {},
        onToggleIpv6: (Boolean) -> Unit = {},
        navigateToIpv6Info: () -> Unit = {},
        navigateToDeviceIpInfo: () -> Unit = {},
    ) {
        setContentWithTheme {
            VpnSettingsScreen(
                state = state,
                navigateToContentBlockersInfo = navigateToContentBlockersInfo,
                navigateToAutoConnectScreen = navigateToAutoConnectScreen,
                navigateToCustomDnsInfo = navigateToCustomDnsInfo,
                navigateToMalwareInfo = navigateToMalwareInfo,
                navigateToObfuscationInfo = navigateToObfuscationInfo,
                navigateToQuantumResistanceInfo = navigateToQuantumResistanceInfo,
                navigateToWireguardPortInfo = navigateToWireguardPortInfo,
                navigateToLocalNetworkSharingInfo = navigateToLocalNetworkSharingInfo,
                navigateToWireguardPortDialog = navigateToWireguardPortDialog,
                navigateToServerIpOverrides = navigateToServerIpOverrides,
                onToggleBlockTrackers = onToggleBlockTrackers,
                onToggleBlockAds = onToggleBlockAds,
                onToggleBlockMalware = onToggleBlockMalware,
                onToggleLocalNetworkSharing = onToggleLocalNetworkSharing,
                onToggleBlockAdultContent = onToggleBlockAdultContent,
                onToggleBlockGambling = onToggleBlockGambling,
                onToggleBlockSocialMedia = onToggleBlockSocialMedia,
                navigateToMtuDialog = navigateToMtuDialog,
                navigateToDns = navigateToDns,
                onToggleDnsClick = onToggleDnsClick,
                onBackClick = onBackClick,
                onSelectObfuscationMode = onSelectObfuscationMode,
                onSelectQuantumResistanceSetting = onSelectQuantumResistanceSetting,
                onWireguardPortSelected = onWireguardPortSelected,
                navigateToShadowSocksSettings = navigateToShadowSocksSettings,
                navigateToUdp2TcpSettings = navigateToUdp2TcpSettings,
                onToggleAutoStartAndConnectOnBoot = onToggleAutoStartAndConnectOnBoot,
                onSelectDeviceIpVersion = onSelectDeviceIpVersion,
                onToggleIpv6 = onToggleIpv6,
                navigateToIpv6Info = navigateToIpv6Info,
                navigateToDeviceIpInfo = navigateToDeviceIpInfo,
            )
        }
    }

    @Test
    fun testDefaultState() =
        composeExtension.use {
            // Arrange
            initScreen(state = VpnSettingsUiState.createDefault())

            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))

            // Assert
            apply {
                onNodeWithText("WireGuard MTU").assertExists()
                onNodeWithText("Default").assertExists()
            }
        }

    @Test
    fun testMtuCustomValue() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    VpnSettingsUiState.createDefault(
                        mtu = Mtu.fromString(VALID_DUMMY_MTU_VALUE).getOrNull()!!
                    )
            )

            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))

            // Assert
            onNodeWithText(VALID_DUMMY_MTU_VALUE).assertExists()
        }

    @Test
    fun testCustomDnsAddressesAndAddButtonVisibleWhenCustomDnsEnabled() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    VpnSettingsUiState.createDefault(
                        isCustomDnsEnabled = true,
                        customDnsItems =
                            listOf(
                                CustomDnsItem(address = DUMMY_DNS_ADDRESS, false, false),
                                CustomDnsItem(address = DUMMY_DNS_ADDRESS_2, false, false),
                                CustomDnsItem(address = DUMMY_DNS_ADDRESS_3, false, false),
                            ),
                    )
            )

            // Assert
            onNodeWithText(DUMMY_DNS_ADDRESS).assertExists()
            onNodeWithText(DUMMY_DNS_ADDRESS_2).assertExists()
            onNodeWithText(DUMMY_DNS_ADDRESS_3).assertExists()
            onNodeWithText("Add a server").assertExists()
        }

    @Test
    fun testCustomDnsAddressesAndAddButtonNotVisibleWhenCustomDnsDisabled() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    VpnSettingsUiState.createDefault(
                        isCustomDnsEnabled = false,
                        customDnsItems =
                            listOf(CustomDnsItem(address = DUMMY_DNS_ADDRESS, false, false)),
                    )
            )
            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))
            // Assert
            onNodeWithText(DUMMY_DNS_ADDRESS).assertDoesNotExist()
            onNodeWithText("Add a server").assertDoesNotExist()
        }

    @Test
    fun testLanWarningNotShownWhenLanTrafficEnabledAndLocalAddressIsUsed() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    VpnSettingsUiState.createDefault(
                        isCustomDnsEnabled = true,
                        isLocalNetworkSharingEnabled = true,
                        customDnsItems =
                            listOf(
                                CustomDnsItem(
                                    address = DUMMY_DNS_ADDRESS,
                                    isLocal = true,
                                    isIpv6 = false,
                                )
                            ),
                    )
            )

            // Assert
            onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
        }

    @Test
    fun testLanWarningNotShowedWhenLanTrafficDisabledAndLocalAddressIsNotUsed() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    VpnSettingsUiState.createDefault(
                        isCustomDnsEnabled = true,
                        customDnsItems =
                            listOf(
                                CustomDnsItem(
                                    address = DUMMY_DNS_ADDRESS,
                                    isLocal = false,
                                    isIpv6 = false,
                                )
                            ),
                    )
            )

            // Assert
            onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
        }

    @Test
    fun testLanWarningNotShowedWhenLanTrafficEnabledAndLocalAddressIsNotUsed() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    VpnSettingsUiState.createDefault(
                        isCustomDnsEnabled = true,
                        customDnsItems =
                            listOf(
                                CustomDnsItem(
                                    address = DUMMY_DNS_ADDRESS,
                                    isLocal = false,
                                    isIpv6 = false,
                                )
                            ),
                    )
            )

            // Assert
            onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
        }

    @Test
    fun testLanWarningShowedWhenAllowLanEnabledAndLocalDnsAddressIsUsed() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    VpnSettingsUiState.createDefault(
                        isCustomDnsEnabled = true,
                        customDnsItems =
                            listOf(
                                CustomDnsItem(
                                    address = DUMMY_DNS_ADDRESS,
                                    isLocal = true,
                                    isIpv6 = false,
                                )
                            ),
                    )
            )

            // Assert
            onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertExists()
        }

    @Test
    fun testShowSelectedTunnelQuantumOption() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    VpnSettingsUiState.createDefault(quantumResistant = QuantumResistantState.On)
            )
            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_QUANTUM_ITEM_OFF_TEST_TAG))

            // Assert
            onNodeWithTagAndText(testTag = LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG, text = "On")
                .assertExists()
        }

    @Test
    fun testSelectTunnelQuantumOption() =
        composeExtension.use {
            // Arrange
            val mockSelectQuantumResistantSettingListener: (QuantumResistantState) -> Unit =
                mockk(relaxed = true)
            initScreen(
                state =
                    VpnSettingsUiState.createDefault(quantumResistant = QuantumResistantState.Auto),
                onSelectQuantumResistanceSetting = mockSelectQuantumResistantSettingListener,
            )
            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_QUANTUM_ITEM_OFF_TEST_TAG))

            // Assert
            onNodeWithTagAndText(testTag = LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG, text = "On")
                .performClick()
            verify(exactly = 1) {
                mockSelectQuantumResistantSettingListener.invoke(QuantumResistantState.On)
            }
        }

    @Test
    fun testShowWireguardPortOptions() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    VpnSettingsUiState.createDefault(
                        selectedWireguardPort = Constraint.Only(Port(53))
                    )
            )

            // Act
            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(
                    hasTestTag(String.format(LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG, 53))
                )

            // Assert
            onNodeWithTagAndText(
                    testTag = String.format(LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG, 51820),
                    text = "51820",
                )
                .assertExists()
        }

    @Test
    fun testSelectWireguardPortOption() =
        composeExtension.use {
            // Arrange
            val mockSelectWireguardPortSelectionListener: (Constraint<Port>) -> Unit =
                mockk(relaxed = true)
            initScreen(
                state =
                    VpnSettingsUiState.createDefault(
                        selectedWireguardPort = Constraint.Only(Port(53))
                    ),
                onWireguardPortSelected = mockSelectWireguardPortSelectionListener,
            )

            // Act
            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(
                    hasTestTag(String.format(LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG, 53))
                )
            onNodeWithTagAndText(
                    testTag = String.format(LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG, 51820),
                    text = "51820",
                )
                .performClick()

            // Assert
            verify(exactly = 1) {
                mockSelectWireguardPortSelectionListener.invoke(Constraint.Only(Port(51820)))
            }
        }

    @Test
    fun testShowWireguardCustomPort() =
        composeExtension.use {
            // Arrange
            initScreen(state = VpnSettingsUiState.createDefault(customWireguardPort = Port(4000)))

            // Act
            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG))

            // Assert
            onNodeWithText("4000").assertExists()
        }

    @Test
    fun testSelectWireguardCustomPort() =
        composeExtension.use {
            // Arrange
            val onWireguardPortSelected: (Constraint<Port>) -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    VpnSettingsUiState.createDefault(
                        selectedWireguardPort = Constraint.Only(Port(4000)),
                        customWireguardPort = Port(4000),
                    ),
                onWireguardPortSelected = onWireguardPortSelected,
            )

            // Act
            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG))
            onNodeWithTag(testTag = LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG).performClick()

            // Assert
            verify { onWireguardPortSelected.invoke(Constraint.Only(Port(4000))) }
        }

    // Navigation Tests

    @Test
    fun testMtuClick() =
        composeExtension.use {
            // Arrange
            val mockedClickHandler: (Mtu?) -> Unit = mockk(relaxed = true)
            initScreen(
                state = VpnSettingsUiState.createDefault(),
                navigateToMtuDialog = mockedClickHandler,
            )

            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))

            // Act
            onNodeWithText("WireGuard MTU").performClick()

            // Assert
            verify { mockedClickHandler.invoke(null) }
        }

    @Test
    fun testClickAddDns() =
        composeExtension.use {
            // Arrange
            val mockedClickHandler: (Int?, String?) -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    VpnSettingsUiState.createDefault(
                        isCustomDnsEnabled = true,
                        customDnsItems = listOf(CustomDnsItem("1.1.1.1", false, false)),
                    ),
                navigateToDns = mockedClickHandler,
            )

            // Act
            onNodeWithText("Add a server").performClick()

            // Assert
            verify { mockedClickHandler.invoke(null, null) }
        }

    @Test
    fun testShowObfuscationInfo() =
        composeExtension.use {
            val mockedNavigateToObfuscationInfo: () -> Unit = mockk(relaxed = true)

            // Arrange
            initScreen(
                state = VpnSettingsUiState.createDefault(),
                navigateToObfuscationInfo = mockedNavigateToObfuscationInfo,
            )

            // Act
            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_WIREGUARD_OBFUSCATION_TITLE_TEST_TAG))
            onNodeWithText("WireGuard obfuscation").performClick()

            // Assert
            verify(exactly = 1) { mockedNavigateToObfuscationInfo() }
        }

    @Test
    fun testShowTunnelQuantumInfo() =
        composeExtension.use {
            val mockedShowTunnelQuantumInfoClick: () -> Unit = mockk(relaxed = true)

            // Arrange
            initScreen(
                state = VpnSettingsUiState.createDefault(),
                navigateToQuantumResistanceInfo = mockedShowTunnelQuantumInfoClick,
            )

            // Act
            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG))
            onNodeWithText("Quantum-resistant tunnel").performClick()

            // Assert
            verify(exactly = 1) { mockedShowTunnelQuantumInfoClick() }
        }

    @Test
    fun testShowWireguardPortInfo() =
        composeExtension.use {
            val mockedClickHandler: (List<PortRange>) -> Unit = mockk(relaxed = true)

            // Arrange
            initScreen(
                state = VpnSettingsUiState.createDefault(),
                navigateToWireguardPortInfo = mockedClickHandler,
            )

            onNodeWithText("WireGuard port").performClick()

            verify(exactly = 1) { mockedClickHandler.invoke(any()) }
        }

    @Test
    fun testShowWireguardCustomPortDialog() =
        composeExtension.use {
            val mockedClickHandler: () -> Unit = mockk(relaxed = true)

            // Arrange
            initScreen(
                state = VpnSettingsUiState.createDefault(),
                navigateToWireguardPortDialog = mockedClickHandler,
            )

            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_WIREGUARD_CUSTOM_PORT_NUMBER_TEST_TAG))
            onNodeWithText("Custom").performClick()

            // Assert
            verify(exactly = 1) { mockedClickHandler.invoke() }
        }

    @Test
    fun testClickWireguardCustomPortMainCell() =
        composeExtension.use {
            // Arrange
            val mockOnShowCustomPortDialog: () -> Unit = mockk(relaxed = true)
            initScreen(
                state = VpnSettingsUiState.createDefault(),
                navigateToWireguardPortDialog = mockOnShowCustomPortDialog,
            )

            // Act
            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG))
            onNodeWithTag(LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG).performClick()

            // Assert
            verify { mockOnShowCustomPortDialog.invoke() }
        }

    @Test
    fun testClickWireguardCustomPortNumberCell() =
        composeExtension.use {
            // Arrange
            val mockOnShowCustomPortDialog: () -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    VpnSettingsUiState.createDefault(
                        selectedWireguardPort = Constraint.Only(Port(4000))
                    ),
                navigateToWireguardPortDialog = mockOnShowCustomPortDialog,
            )

            // Act
            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG))
            onNodeWithTag(testTag = LAZY_LIST_WIREGUARD_CUSTOM_PORT_NUMBER_TEST_TAG).performClick()

            // Assert
            verify { mockOnShowCustomPortDialog.invoke() }
        }

    @Test
    fun ensureConnectOnStartIsShownWhenSystemVpnSettingsAvailableIsFalse() =
        composeExtension.use {
            // Arrange
            initScreen(state = VpnSettingsUiState.createDefault(systemVpnSettingsAvailable = false))

            // Assert
            onNodeWithText("Connect on device start-up").assertExists()
        }

    @Test
    fun whenClickingOnConnectOnStartShouldCallOnToggleAutoStartAndConnectOnBoot() =
        composeExtension.use {
            // Arrange
            val mockOnToggleAutoStartAndConnectOnBoot: (Boolean) -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    VpnSettingsUiState.createDefault(
                        systemVpnSettingsAvailable = false,
                        autoStartAndConnectOnBoot = false,
                    ),
                onToggleAutoStartAndConnectOnBoot = mockOnToggleAutoStartAndConnectOnBoot,
            )

            // Act
            onNodeWithText("Connect on device start-up").performClick()

            // Assert
            verify { mockOnToggleAutoStartAndConnectOnBoot.invoke(true) }
        }

    companion object {
        private const val LOCAL_DNS_SERVER_WARNING =
            "The local DNS server will not work unless you enable " +
                "\"Local Network Sharing\" under VPN settings."
        private const val VALID_DUMMY_MTU_VALUE = "1337"
        private const val DUMMY_DNS_ADDRESS = "0.0.0.1"
        private const val DUMMY_DNS_ADDRESS_2 = "0.0.0.2"
        private const val DUMMY_DNS_ADDRESS_3 = "0.0.0.3"
    }
}
