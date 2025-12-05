package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assert
import androidx.compose.ui.test.hasAnyChild
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.isOn
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
import net.mullvad.mullvadvpn.compose.state.CustomDnsItem
import net.mullvad.mullvadvpn.compose.state.VpnSettingsUiState
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_LAST_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_QUANTUM_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_VPN_SETTINGS_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SWITCH_TEST_TAG
import net.mullvad.mullvadvpn.onNodeWithContentDescriptionAndParentTag
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.toLc
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

    private fun createDefaultUiState(
        mtu: Mtu? = null,
        isLocalNetworkSharingEnabled: Boolean = false,
        isCustomDnsEnabled: Boolean = false,
        customDnsItems: List<CustomDnsItem> = emptyList(),
        contentBlockersOptions: DefaultDnsOptions = DefaultDnsOptions(),
        obfuscationMode: ObfuscationMode = ObfuscationMode.Auto,
        quantumResistant: QuantumResistantState = QuantumResistantState.On,
        systemVpnSettingsAvailable: Boolean = true,
        autoStartAndConnectOnBoot: Boolean = false,
        deviceIpVersion: Constraint<IpVersion> = Constraint.Any,
        isIpv6Enabled: Boolean = true,
        isContentBlockersExpanded: Boolean = false,
        isModal: Boolean = false,
        isScrollToFeatureEnabled: Boolean = true,
    ) =
        VpnSettingsUiState.from(
            mtu = mtu,
            isLocalNetworkSharingEnabled = isLocalNetworkSharingEnabled,
            isCustomDnsEnabled = isCustomDnsEnabled,
            customDnsItems = customDnsItems,
            contentBlockersOptions = contentBlockersOptions,
            obfuscationMode = obfuscationMode,
            quantumResistant = quantumResistant,
            systemVpnSettingsAvailable = systemVpnSettingsAvailable,
            autoStartAndConnectOnBoot = autoStartAndConnectOnBoot,
            deviceIpVersion = deviceIpVersion,
            isIpv6Enabled = isIpv6Enabled,
            isContentBlockersExpanded = isContentBlockersExpanded,
            isModal = isModal,
            isScrollToFeatureEnabled = isScrollToFeatureEnabled,
        )

    private fun ComposeContext.initScreen(
        state: Lc<Boolean, VpnSettingsUiState> = createDefaultUiState().toLc(),
        navigateToContentBlockersInfo: () -> Unit = {},
        navigateToAutoConnectScreen: () -> Unit = {},
        navigateToCustomDnsInfo: () -> Unit = {},
        navigateToMalwareInfo: () -> Unit = {},
        navigateToQuantumResistanceInfo: () -> Unit = {},
        navigateToLocalNetworkSharingInfo: () -> Unit = {},
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
        onSelectQuantumResistanceSetting: (Boolean) -> Unit = {},
        onToggleAutoStartAndConnectOnBoot: (Boolean) -> Unit = {},
        onSelectDeviceIpVersion: (Constraint<IpVersion>) -> Unit = {},
        onToggleIpv6: (Boolean) -> Unit = {},
        navigateToIpv6Info: () -> Unit = {},
        onToggleDnsContentBlockers: () -> Unit = {},
        navigateToDeviceIpInfo: () -> Unit = {},
        navigateToConnectOnDeviceOnStartUpInfo: () -> Unit = {},
        navigateToAntiCensorship: () -> Unit = {},
    ) {
        setContentWithTheme {
            VpnSettingsScreen(
                state = state,
                navigateToContentBlockersInfo = navigateToContentBlockersInfo,
                navigateToAutoConnectScreen = navigateToAutoConnectScreen,
                navigateToCustomDnsInfo = navigateToCustomDnsInfo,
                navigateToMalwareInfo = navigateToMalwareInfo,
                navigateToQuantumResistanceInfo = navigateToQuantumResistanceInfo,
                navigateToLocalNetworkSharingInfo = navigateToLocalNetworkSharingInfo,
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
                onSelectQuantumResistanceSetting = onSelectQuantumResistanceSetting,
                onToggleAutoStartAndConnectOnBoot = onToggleAutoStartAndConnectOnBoot,
                onSelectDeviceIpVersion = onSelectDeviceIpVersion,
                onToggleIpv6 = onToggleIpv6,
                navigateToIpv6Info = navigateToIpv6Info,
                onToggleContentBlockersExpanded = onToggleDnsContentBlockers,
                navigateToDeviceIpInfo = navigateToDeviceIpInfo,
                navigateToConnectOnDeviceOnStartUpInfo = navigateToConnectOnDeviceOnStartUpInfo,
                navigateToAntiCensorship = navigateToAntiCensorship,
                initialScrollToFeature = null,
            )
        }
    }

    @Test
    fun testDefaultState() =
        composeExtension.use {
            // Arrange
            initScreen()

            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))

            // Assert
            apply {
                onNodeWithText("MTU").assertExists()
                onNodeWithText("Value: Default").assertExists()
            }
        }

    @Test
    fun testMtuCustomValue() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    createDefaultUiState(mtu = Mtu.fromString(VALID_DUMMY_MTU_VALUE).getOrNull()!!)
                        .toLc()
            )

            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))

            // Assert
            onNodeWithText("Value: $VALID_DUMMY_MTU_VALUE").assertExists()
        }

    @Test
    fun testCustomDnsAddressesAndAddButtonVisibleWhenCustomDnsEnabled() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    createDefaultUiState(
                            isCustomDnsEnabled = true,
                            customDnsItems =
                                listOf(
                                    CustomDnsItem(address = DUMMY_DNS_ADDRESS, false, false),
                                    CustomDnsItem(address = DUMMY_DNS_ADDRESS_2, false, false),
                                    CustomDnsItem(address = DUMMY_DNS_ADDRESS_3, false, false),
                                ),
                        )
                        .toLc()
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
                    createDefaultUiState(
                            isCustomDnsEnabled = false,
                            customDnsItems =
                                listOf(CustomDnsItem(address = DUMMY_DNS_ADDRESS, false, false)),
                        )
                        .toLc()
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
                    createDefaultUiState(
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
                        .toLc()
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
                    createDefaultUiState(
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
                        .toLc()
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
                    createDefaultUiState(
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
                        .toLc()
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
                    createDefaultUiState(
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
                        .toLc()
            )

            // Assert
            onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertExists()
        }

    @Test
    fun testShowSelectedTunnelQuantumOption() =
        composeExtension.use {
            // Arrange
            initScreen(
                state = createDefaultUiState(quantumResistant = QuantumResistantState.On).toLc()
            )
            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_QUANTUM_ITEM_TEST_TAG))

            // Assert
            onNodeWithTag(testTag = LAZY_LIST_QUANTUM_ITEM_TEST_TAG)
                .assertExists()
                .assert(hasAnyChild(hasTestTag(SWITCH_TEST_TAG).and(isOn())))
        }

    @Test
    fun testSelectTunnelQuantumOption() =
        composeExtension.use {
            // Arrange
            val mockSelectQuantumResistantSettingListener: (Boolean) -> Unit = mockk(relaxed = true)
            initScreen(
                state = createDefaultUiState(quantumResistant = QuantumResistantState.Off).toLc(),
                onSelectQuantumResistanceSetting = mockSelectQuantumResistantSettingListener,
            )
            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_QUANTUM_ITEM_TEST_TAG))

            // Assert
            onNodeWithTag(testTag = LAZY_LIST_QUANTUM_ITEM_TEST_TAG).performClick()
            verify(exactly = 1) { mockSelectQuantumResistantSettingListener.invoke(true) }
        }

    // Navigation Tests

    @Test
    fun testMtuClick() =
        composeExtension.use {
            // Arrange
            val mockedClickHandler: (Mtu?) -> Unit = mockk(relaxed = true)
            initScreen(
                state = createDefaultUiState().toLc(),
                navigateToMtuDialog = mockedClickHandler,
            )

            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))

            // Act
            onNodeWithText("MTU").performClick()

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
                    createDefaultUiState(
                            isCustomDnsEnabled = true,
                            customDnsItems = listOf(CustomDnsItem("1.1.1.1", false, false)),
                        )
                        .toLc(),
                navigateToDns = mockedClickHandler,
            )

            // Act
            onNodeWithText("Add a server").performClick()

            // Assert
            verify { mockedClickHandler.invoke(null, null) }
        }

    @Test
    fun testShowTunnelQuantumInfo() =
        composeExtension.use {
            val mockedShowTunnelQuantumInfoClick: () -> Unit = mockk(relaxed = true)

            // Arrange
            initScreen(
                state = createDefaultUiState().toLc(),
                navigateToQuantumResistanceInfo = mockedShowTunnelQuantumInfoClick,
            )

            // Act
            onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_QUANTUM_ITEM_TEST_TAG))
            onNodeWithContentDescriptionAndParentTag(
                    "More information",
                    LAZY_LIST_QUANTUM_ITEM_TEST_TAG,
                )
                .performClick()

            // Assert
            verify(exactly = 1) { mockedShowTunnelQuantumInfoClick() }
        }

    @Test
    fun ensureConnectOnStartIsShownWhenSystemVpnSettingsAvailableIsFalse() =
        composeExtension.use {
            // Arrange
            initScreen(state = createDefaultUiState(systemVpnSettingsAvailable = false).toLc())

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
                    createDefaultUiState(
                            systemVpnSettingsAvailable = false,
                            autoStartAndConnectOnBoot = false,
                        )
                        .toLc(),
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
