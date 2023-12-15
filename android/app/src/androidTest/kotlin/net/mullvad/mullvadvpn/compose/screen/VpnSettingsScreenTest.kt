package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollToNode
import de.mannodermaus.junit5.compose.createComposeExtension
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
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class VpnSettingsScreenTest {
    @JvmField @RegisterExtension val composeExtension = createComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun testDefaultState() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                VpnSettingsScreen(
                    uiState = VpnSettingsUiState.createDefault(),
                )
            }

            apply { onNodeWithText("Auto-connect").assertExists() }

            onNodeWithTag(LAZY_LIST_TEST_TAG)
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
            setContentWithTheme {
                VpnSettingsScreen(
                    uiState = VpnSettingsUiState.createDefault(mtu = VALID_DUMMY_MTU_VALUE),
                )
            }

            onNodeWithTag(LAZY_LIST_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))

            // Assert
            onNodeWithText(VALID_DUMMY_MTU_VALUE).assertExists()
        }

    @Test
    fun testCustomDnsAddressesAndAddButtonVisibleWhenCustomDnsEnabled() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
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
            onNodeWithText(DUMMY_DNS_ADDRESS).assertExists()
            onNodeWithText(DUMMY_DNS_ADDRESS_2).assertExists()
            onNodeWithText(DUMMY_DNS_ADDRESS_3).assertExists()
            onNodeWithText("Add a server").assertExists()
        }

    @Test
    fun testCustomDnsAddressesAndAddButtonNotVisibleWhenCustomDnsDisabled() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                VpnSettingsScreen(
                    uiState =
                        VpnSettingsUiState.createDefault(
                            isCustomDnsEnabled = false,
                            customDnsItems =
                                listOf(CustomDnsItem(address = DUMMY_DNS_ADDRESS, false))
                        ),
                )
            }
            onNodeWithTag(LAZY_LIST_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))
            // Assert
            onNodeWithText(DUMMY_DNS_ADDRESS).assertDoesNotExist()
            onNodeWithText("Add a server").assertDoesNotExist()
        }

    @Test
    fun testLanWarningNotShownWhenLanTrafficEnabledAndLocalAddressIsUsed() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
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
            onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
        }

    @Test
    fun testLanWarningNotShowedWhenLanTrafficDisabledAndLocalAddressIsNotUsed() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
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
            onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
        }

    @Test
    fun testLanWarningNotShowedWhenLanTrafficEnabledAndLocalAddressIsNotUsed() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
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
            onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
        }

    @Test
    fun testLanWarningShowedWhenAllowLanEnabledAndLocalDnsAddressIsUsed() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
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
            onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertExists()
        }

    @Test
    fun testShowSelectedTunnelQuantumOption() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                VpnSettingsScreen(
                    uiState =
                        VpnSettingsUiState.createDefault(
                            quantumResistant = QuantumResistantState.On
                        ),
                )
            }
            onNodeWithTag(LAZY_LIST_TEST_TAG)
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
            setContentWithTheme {
                VpnSettingsScreen(
                    uiState =
                        VpnSettingsUiState.createDefault(
                            quantumResistant = QuantumResistantState.Auto,
                        ),
                    onSelectQuantumResistanceSetting = mockSelectQuantumResistantSettingListener,
                )
            }
            onNodeWithTag(LAZY_LIST_TEST_TAG)
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
            setContentWithTheme {
                VpnSettingsScreen(
                    uiState =
                        VpnSettingsUiState.createDefault(
                            selectedWireguardPort = Constraint.Only(Port(53))
                        ),
                )
            }

            // Act
            onNodeWithTag(LAZY_LIST_TEST_TAG)
                .performScrollToNode(
                    hasTestTag(String.format(LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG, 53))
                )

            // Assert
            onNodeWithTagAndText(
                    testTag = String.format(LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG, 51820),
                    text = "51820"
                )
                .assertExists()
        }

    @Test
    fun testSelectWireguardPortOption() =
        composeExtension.use {
            // Arrange
            val mockSelectWireguardPortSelectionListener: (Constraint<Port>) -> Unit =
                mockk(relaxed = true)
            setContentWithTheme {
                VpnSettingsScreen(
                    uiState =
                        VpnSettingsUiState.createDefault(
                            selectedWireguardPort = Constraint.Only(Port(53))
                        ),
                    onWireguardPortSelected = mockSelectWireguardPortSelectionListener,
                )
            }

            // Act
            onNodeWithTag(LAZY_LIST_TEST_TAG)
                .performScrollToNode(
                    hasTestTag(String.format(LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG, 53))
                )
            onNodeWithTagAndText(
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
    fun testShowWireguardCustomPort() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                VpnSettingsScreen(
                    uiState =
                        VpnSettingsUiState.createDefault(
                            customWireguardPort = Constraint.Only(Port(4000))
                        ),
                )
            }

            // Act
            onNodeWithTag(LAZY_LIST_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG))

            // Assert
            onNodeWithText("4000").assertExists()
        }

    @Test
    fun testSelectWireguardCustomPort() =
        composeExtension.use {
            // Arrange
            val onWireguardPortSelected: (Constraint<Port>) -> Unit = mockk(relaxed = true)
            setContentWithTheme {
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
            onNodeWithTag(LAZY_LIST_TEST_TAG)
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
            val mockedClickHandler: (Int?) -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                VpnSettingsScreen(
                    uiState = VpnSettingsUiState.createDefault(),
                    navigateToMtuDialog = mockedClickHandler
                )
            }

            onNodeWithTag(LAZY_LIST_TEST_TAG)
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
            setContentWithTheme {
                VpnSettingsScreen(
                    uiState = VpnSettingsUiState.createDefault(isCustomDnsEnabled = true),
                    navigateToDns = mockedClickHandler
                )
            }

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
            setContentWithTheme {
                VpnSettingsScreen(
                    uiState = VpnSettingsUiState.createDefault(),
                    navigateToQuantumResistanceInfo = mockedShowTunnelQuantumInfoClick
                )
            }

            // Act

            onNodeWithTag(LAZY_LIST_TEST_TAG)
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
            setContentWithTheme {
                VpnSettingsScreen(
                    uiState = VpnSettingsUiState.createDefault(),
                    navigateToWireguardPortInfo = mockedClickHandler
                )
            }

            onNodeWithText("WireGuard port").performClick()

            verify(exactly = 1) { mockedClickHandler.invoke(any()) }
        }

    @Test
    fun testShowWireguardCustomPortDialog() =
        composeExtension.use {
            val mockedClickHandler: () -> Unit = mockk(relaxed = true)

            // Arrange
            setContentWithTheme {
                VpnSettingsScreen(
                    uiState = VpnSettingsUiState.createDefault(),
                    navigateToWireguardPortDialog = mockedClickHandler
                )
            }

            onNodeWithTag(LAZY_LIST_TEST_TAG)
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
            setContentWithTheme {
                VpnSettingsScreen(
                    uiState = VpnSettingsUiState.createDefault(),
                    navigateToWireguardPortDialog = mockOnShowCustomPortDialog
                )
            }

            // Act
            onNodeWithTag(LAZY_LIST_TEST_TAG)
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
            setContentWithTheme {
                VpnSettingsScreen(
                    uiState =
                        VpnSettingsUiState.createDefault(
                            selectedWireguardPort = Constraint.Only(Port(4000))
                        ),
                    navigateToWireguardPortDialog = mockOnShowCustomPortDialog
                )
            }

            // Act
            onNodeWithTag(LAZY_LIST_TEST_TAG)
                .performScrollToNode(hasTestTag(LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG))
            onNodeWithTag(testTag = LAZY_LIST_WIREGUARD_CUSTOM_PORT_NUMBER_TEST_TAG).performClick()

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
