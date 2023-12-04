package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollToNode
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.VpnSettingsUiState
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_LAST_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_QUANTUM_ITEM_OFF_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_WIREGUARD_CUSTOM_PORT_NUMBER_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.Port
import net.mullvad.mullvadvpn.model.PortRange
import net.mullvad.mullvadvpn.model.QuantumResistantState
import net.mullvad.mullvadvpn.onNodeWithTagAndText
import net.mullvad.mullvadvpn.viewmodel.CustomDnsItem
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class VpnSettingsScreenTest {
    @get:Rule val composeTestRule = createComposeRule()

    @Before
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun testDefaultState() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState = VpnSettingsUiState.createDefault(),
            )
        }

        composeTestRule.apply { onNodeWithText("Auto-connect").assertExists() }

        composeTestRule
            .onNodeWithTag(LAZY_LIST_TEST_TAG)
            .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))

        // Assert
        composeTestRule.apply {
            onNodeWithText("WireGuard MTU").assertExists()
            onNodeWithText("Default").assertExists()
        }
    }

    @Test
    fun testMtuCustomValue() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState = VpnSettingsUiState.createDefault(mtu = VALID_DUMMY_MTU_VALUE),
            )
        }

        composeTestRule
            .onNodeWithTag(LAZY_LIST_TEST_TAG)
            .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))

        // Assert
        composeTestRule.onNodeWithText(VALID_DUMMY_MTU_VALUE).assertExists()
    }

    @Test
    fun testCustomDnsAddressesAndAddButtonVisibleWhenCustomDnsEnabled() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        isCustomDnsEnabled = true,
                        customDnsItems =
                            listOf(
                                CustomDnsItem(address = DUMMY_DNS_ADDRESS, false),
                                CustomDnsItem(address = DUMMY_DNS_ADDRESS_2, false),
                                CustomDnsItem(address = DUMMY_DNS_ADDRESS_3, false)
                            )
                    ),
            )
        }

        // Assert
        composeTestRule.apply {
            onNodeWithText(DUMMY_DNS_ADDRESS).assertExists()
            onNodeWithText(DUMMY_DNS_ADDRESS_2).assertExists()
            onNodeWithText(DUMMY_DNS_ADDRESS_3).assertExists()
            onNodeWithText("Add a server").assertExists()
        }
    }

    @Test
    fun testCustomDnsAddressesAndAddButtonNotVisibleWhenCustomDnsDisabled() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        isCustomDnsEnabled = false,
                        customDnsItems = listOf(CustomDnsItem(address = DUMMY_DNS_ADDRESS, false))
                    ),
            )
        }
        composeTestRule
            .onNodeWithTag(LAZY_LIST_TEST_TAG)
            .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))
        // Assert
        composeTestRule.onNodeWithText(DUMMY_DNS_ADDRESS).assertDoesNotExist()
        composeTestRule.onNodeWithText("Add a server").assertDoesNotExist()
    }

    @Test
    fun testLanWarningNotShownWhenLanTrafficEnabledAndLocalAddressIsUsed() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        isCustomDnsEnabled = true,
                        isLocalNetworkSharingEnabled = true,
                        customDnsItems =
                            listOf(CustomDnsItem(address = DUMMY_DNS_ADDRESS, isLocal = true))
                    ),
            )
        }

        // Assert
        composeTestRule.onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
    }

    @Test
    fun testLanWarningNotShowedWhenLanTrafficDisabledAndLocalAddressIsNotUsed() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        isCustomDnsEnabled = true,
                        customDnsItems =
                            listOf(CustomDnsItem(address = DUMMY_DNS_ADDRESS, isLocal = false))
                    ),
            )
        }

        // Assert
        composeTestRule.onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
    }

    @Test
    fun testLanWarningNotShowedWhenLanTrafficEnabledAndLocalAddressIsNotUsed() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        isCustomDnsEnabled = true,
                        customDnsItems =
                            listOf(CustomDnsItem(address = DUMMY_DNS_ADDRESS, isLocal = false))
                    ),
            )
        }

        // Assert
        composeTestRule.onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
    }

    @Test
    fun testLanWarningShowedWhenAllowLanEnabledAndLocalDnsAddressIsUsed() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        isCustomDnsEnabled = true,
                        customDnsItems =
                            listOf(CustomDnsItem(address = DUMMY_DNS_ADDRESS, isLocal = true))
                    ),
            )
        }

        // Assert
        composeTestRule.apply {
            onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertExists()
        }
    }

    @Test
    fun testShowSelectedTunnelQuantumOption() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(quantumResistant = QuantumResistantState.On),
            )
        }
        composeTestRule
            .onNodeWithTag(LAZY_LIST_TEST_TAG)
            .performScrollToNode(hasTestTag(LAZY_LIST_QUANTUM_ITEM_OFF_TEST_TAG))

        // Assert
        composeTestRule
            .onNodeWithTagAndText(testTag = LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG, text = "On")
            .assertExists()
    }

    @Test
    fun testSelectTunnelQuantumOption() {
        // Arrange
        val mockSelectQuantumResistantSettingListener: (QuantumResistantState) -> Unit =
            mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        quantumResistant = QuantumResistantState.Auto,
                    ),
                onSelectQuantumResistanceSetting = mockSelectQuantumResistantSettingListener
            )
        }
        composeTestRule
            .onNodeWithTag(LAZY_LIST_TEST_TAG)
            .performScrollToNode(hasTestTag(LAZY_LIST_QUANTUM_ITEM_OFF_TEST_TAG))

        // Assert
        composeTestRule
            .onNodeWithTagAndText(testTag = LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG, text = "On")
            .performClick()
        verify(exactly = 1) {
            mockSelectQuantumResistantSettingListener.invoke(QuantumResistantState.On)
        }
    }

    @Test
    fun testShowWireguardPortOptions() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        selectedWireguardPort = Constraint.Only(Port(53))
                    ),
            )
        }

        // Act
        composeTestRule
            .onNodeWithTag(LAZY_LIST_TEST_TAG)
            .performScrollToNode(
                hasTestTag(String.format(LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG, 53))
            )

        // Assert
        composeTestRule
            .onNodeWithTagAndText(
                testTag = String.format(LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG, 51820),
                text = "51820"
            )
            .assertExists()
    }

    @Test
    fun testSelectWireguardPortOption() {
        // Arrange
        val mockSelectWireguardPortSelectionListener: (Constraint<Port>) -> Unit =
            mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        selectedWireguardPort = Constraint.Only(Port(53))
                    ),
                onWireguardPortSelected = mockSelectWireguardPortSelectionListener
            )
        }

        // Act
        composeTestRule
            .onNodeWithTag(LAZY_LIST_TEST_TAG)
            .performScrollToNode(
                hasTestTag(String.format(LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG, 53))
            )
        composeTestRule
            .onNodeWithTagAndText(
                testTag = String.format(LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG, 51820),
                text = "51820"
            )
            .performClick()

        // Assert
        verify(exactly = 1) {
            mockSelectWireguardPortSelectionListener.invoke(Constraint.Only(Port(51820)))
        }
    }

    @Test
    fun testShowWireguardCustomPort() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        customWireguardPort = Constraint.Only(Port(4000))
                    ),
            )
        }

        // Act
        composeTestRule
            .onNodeWithTag(LAZY_LIST_TEST_TAG)
            .performScrollToNode(hasTestTag(LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG))

        // Assert
        composeTestRule.onNodeWithText("4000").assertExists()
    }

    @Test
    fun testSelectWireguardCustomPort() {
        // Arrange
        val onWireguardPortSelected: (Constraint<Port>) -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        selectedWireguardPort = Constraint.Only(Port(4000)),
                        customWireguardPort = Constraint.Only(Port(4000))
                    ),
                onWireguardPortSelected = onWireguardPortSelected
            )
        }

        // Act
        composeTestRule
            .onNodeWithTag(LAZY_LIST_TEST_TAG)
            .performScrollToNode(hasTestTag(LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG))
        composeTestRule
            .onNodeWithTag(testTag = LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG)
            .performClick()

        // Assert
        verify { onWireguardPortSelected.invoke(Constraint.Only(Port(4000))) }
    }

    // Navigation Tests

    @Test
    fun testMtuClick() {
        // Arrange
        val mockedClickHandler: (Int?) -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState = VpnSettingsUiState.createDefault(),
                navigateToMtuDialog = mockedClickHandler
            )
        }

        composeTestRule
            .onNodeWithTag(LAZY_LIST_TEST_TAG)
            .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))

        // Act
        composeTestRule.onNodeWithText("WireGuard MTU").performClick()

        // Assert
        verify { mockedClickHandler.invoke(null) }
    }

    @Test
    fun testClickAddDns() {
        // Arrange
        val mockedClickHandler: (Int?, String?) -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState = VpnSettingsUiState.createDefault(isCustomDnsEnabled = true),
                navigateToDns = mockedClickHandler
            )
        }

        // Act
        composeTestRule.onNodeWithText("Add a server").performClick()

        // Assert
        verify { mockedClickHandler.invoke(null, null) }
    }

    @Test
    fun testShowTunnelQuantumInfo() {
        val mockedShowTunnelQuantumInfoClick: () -> Unit = mockk(relaxed = true)

        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState = VpnSettingsUiState.createDefault(),
                navigateToQuantumResistanceInfo = mockedShowTunnelQuantumInfoClick
            )
        }

        // Act
        composeTestRule
            .onNodeWithTag(LAZY_LIST_TEST_TAG)
            .performScrollToNode(hasTestTag(LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG))
        composeTestRule.onNodeWithText("Quantum-resistant tunnel").performClick()

        // Assert
        verify(exactly = 1) { mockedShowTunnelQuantumInfoClick() }
    }

    @Test
    fun testShowWireguardPortInfo() {
        val mockedClickHandler: (List<PortRange>) -> Unit = mockk(relaxed = true)

        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState = VpnSettingsUiState.createDefault(),
                navigateToWireguardPortInfo = mockedClickHandler
            )
        }

        composeTestRule.onNodeWithText("WireGuard port").performClick()

        verify(exactly = 1) { mockedClickHandler.invoke(any()) }
    }

    @Test
    fun testShowWireguardCustomPortDialog() {
        val mockedClickHandler: () -> Unit = mockk(relaxed = true)

        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState = VpnSettingsUiState.createDefault(),
                navigateToWireguardPortDialog = mockedClickHandler
            )
        }

        composeTestRule
            .onNodeWithTag(LAZY_LIST_TEST_TAG)
            .performScrollToNode(hasTestTag(LAZY_LIST_WIREGUARD_CUSTOM_PORT_NUMBER_TEST_TAG))
        composeTestRule.onNodeWithText("Custom").performClick()

        // Assert
        verify(exactly = 1) { mockedClickHandler.invoke() }
    }

    @Test
    fun testClickWireguardCustomPortMainCell() {
        // Arrange
        val mockOnShowCustomPortDialog: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState = VpnSettingsUiState.createDefault(),
                navigateToWireguardPortDialog = mockOnShowCustomPortDialog
            )
        }

        // Act
        composeTestRule
            .onNodeWithTag(LAZY_LIST_TEST_TAG)
            .performScrollToNode(hasTestTag(LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG))
        composeTestRule.onNodeWithTag(LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG).performClick()

        // Assert
        verify { mockOnShowCustomPortDialog.invoke() }
    }

    @Test
    fun testClickWireguardCustomPortNumberCell() {
        // Arrange
        val mockOnShowCustomPortDialog: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        selectedWireguardPort = Constraint.Only(Port(4000))
                    ),
                navigateToWireguardPortDialog = mockOnShowCustomPortDialog
            )
        }

        // Act
        composeTestRule
            .onNodeWithTag(LAZY_LIST_TEST_TAG)
            .performScrollToNode(hasTestTag(LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG))
        composeTestRule
            .onNodeWithTag(testTag = LAZY_LIST_WIREGUARD_CUSTOM_PORT_NUMBER_TEST_TAG)
            .performClick()

        // Assert
        verify { mockOnShowCustomPortDialog.invoke() }
    }

    companion object {
        private const val LOCAL_DNS_SERVER_WARNING =
            "The local DNS server will not work unless you enable " +
                "\"Local Network Sharing\" under Preferences."
        private const val VALID_DUMMY_MTU_VALUE = "1337"
        private const val DUMMY_DNS_ADDRESS = "0.0.0.1"
        private const val DUMMY_DNS_ADDRESS_2 = "0.0.0.2"
        private const val DUMMY_DNS_ADDRESS_3 = "0.0.0.3"
    }
}
