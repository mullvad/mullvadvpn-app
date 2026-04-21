package net.mullvad.mullvadvpn.feature.vpnsettings.impl

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assert
import androidx.compose.ui.test.hasAnyChild
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.isOn
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollToNode
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.feature.vpnsettings.impl.util.onNodeWithContentDescriptionAndParentTag
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_LAST_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_QUANTUM_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_VPN_SETTINGS_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SWITCH_TEST_TAG
import net.mullvad.mullvadvpn.screen.test.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.screen.test.setContentWithTheme
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
        obfuscationMode: ObfuscationMode = ObfuscationMode.Auto,
        quantumResistant: QuantumResistantState = QuantumResistantState.On,
        systemVpnSettingsAvailable: Boolean = true,
        autoStartAndConnectOnBoot: Boolean = false,
        deviceIpVersion: Constraint<IpVersion> = Constraint.Any,
        isIpv6Enabled: Boolean = true,
        isModal: Boolean = false,
    ) =
        VpnSettingsUiState.from(
            mtu = mtu,
            isLocalNetworkSharingEnabled = isLocalNetworkSharingEnabled,
            obfuscationMode = obfuscationMode,
            quantumResistant = quantumResistant,
            systemVpnSettingsAvailable = systemVpnSettingsAvailable,
            autoStartAndConnectOnBoot = autoStartAndConnectOnBoot,
            deviceIpVersion = deviceIpVersion,
            isIpv6Enabled = isIpv6Enabled,
            isModal = isModal,
        )

    private fun ComposeContext.initScreen(
        state: Lc<Boolean, VpnSettingsUiState> = createDefaultUiState().toLc(),
        navigateToAutoConnectScreen: () -> Unit = {},
        navigateToQuantumResistanceInfo: () -> Unit = {},
        navigateToLocalNetworkSharingInfo: () -> Unit = {},
        navigateToServerIpOverrides: () -> Unit = {},
        onToggleLocalNetworkSharing: (Boolean) -> Unit = {},
        navigateToMtuDialog: (mtu: Mtu?) -> Unit = {},
        navigateToDns: () -> Unit = {},
        onBackClick: () -> Unit = {},
        onSelectQuantumResistanceSetting: (Boolean) -> Unit = {},
        onToggleAutoStartAndConnectOnBoot: (Boolean) -> Unit = {},
        onSelectDeviceIpVersion: (Constraint<IpVersion>) -> Unit = {},
        onToggleIpv6: (Boolean) -> Unit = {},
        navigateToIpv6Info: () -> Unit = {},
        navigateToDeviceIpInfo: () -> Unit = {},
        navigateToConnectOnDeviceOnStartUpInfo: () -> Unit = {},
        navigateToAntiCensorship: () -> Unit = {},
    ) {
        setContentWithTheme {
            VpnSettingsScreen(
                state = state,
                navigateToAutoConnectScreen = navigateToAutoConnectScreen,
                navigateToQuantumResistanceInfo = navigateToQuantumResistanceInfo,
                navigateToLocalNetworkSharingInfo = navigateToLocalNetworkSharingInfo,
                navigateToServerIpOverrides = navigateToServerIpOverrides,
                onToggleLocalNetworkSharing = onToggleLocalNetworkSharing,
                navigateToMtuDialog = navigateToMtuDialog,
                navigateToDns = navigateToDns,
                onBackClick = onBackClick,
                onSelectQuantumResistanceSetting = onSelectQuantumResistanceSetting,
                onToggleAutoStartAndConnectOnBoot = onToggleAutoStartAndConnectOnBoot,
                onSelectDeviceIpVersion = onSelectDeviceIpVersion,
                onToggleIpv6 = onToggleIpv6,
                navigateToIpv6Info = navigateToIpv6Info,
                navigateToDeviceIpInfo = navigateToDeviceIpInfo,
                navigateToConnectOnDeviceOnStartUpInfo = navigateToConnectOnDeviceOnStartUpInfo,
                navigateToAntiCensorship = navigateToAntiCensorship,
                initialScrollToFeature = null,
            )
        }
    }

    @Test
    fun testDefaultState() = composeExtension.use {
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
    fun testMtuCustomValue() = composeExtension.use {
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
    fun testShowSelectedTunnelQuantumOption() = composeExtension.use {
        // Arrange
        initScreen(state = createDefaultUiState(quantumResistant = QuantumResistantState.On).toLc())
        onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
            .performScrollToNode(hasTestTag(LAZY_LIST_QUANTUM_ITEM_TEST_TAG))

        // Assert
        onNodeWithTag(testTag = LAZY_LIST_QUANTUM_ITEM_TEST_TAG)
            .assertExists()
            .assert(hasAnyChild(hasTestTag(SWITCH_TEST_TAG).and(isOn())))
    }

    @Test
    fun testSelectTunnelQuantumOption() = composeExtension.use {
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
    fun testMtuClick() = composeExtension.use {
        // Arrange
        val mockedClickHandler: (Mtu?) -> Unit = mockk(relaxed = true)
        initScreen(state = createDefaultUiState().toLc(), navigateToMtuDialog = mockedClickHandler)

        onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
            .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))

        // Act
        onNodeWithText("MTU").performClick()

        // Assert
        verify { mockedClickHandler.invoke(null) }
    }

    @Test
    fun testShowTunnelQuantumInfo() = composeExtension.use {
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
    fun ensureConnectOnStartIsShownWhenSystemVpnSettingsAvailableIsFalse() = composeExtension.use {
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
        private const val VALID_DUMMY_MTU_VALUE = "1337"
    }
}
