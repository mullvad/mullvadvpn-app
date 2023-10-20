package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollToNode
import androidx.compose.ui.test.performTextInput
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.VpnSettingsDialog
import net.mullvad.mullvadvpn.compose.state.VpnSettingsUiState
import net.mullvad.mullvadvpn.compose.test.CUSTOM_PORT_DIALOG_INPUT_TEST_TAG
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
import net.mullvad.mullvadvpn.viewmodel.StagedDns
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
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
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
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        composeTestRule
            .onNodeWithTag(LAZY_LIST_TEST_TAG)
            .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))

        // Assert
        composeTestRule.onNodeWithText(VALID_DUMMY_MTU_VALUE).assertExists()
    }

    @Test
    fun testMtuClick() {
        // Arrange
        val mockedClickHandler: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState = VpnSettingsUiState.createDefault(),
                onMtuCellClick = mockedClickHandler,
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        composeTestRule
            .onNodeWithTag(LAZY_LIST_TEST_TAG)
            .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))

        // Act
        composeTestRule.onNodeWithText("WireGuard MTU").performClick()

        // Assert
        verify { mockedClickHandler.invoke() }
    }

    @Test
    fun testMtuDialogWithDefaultValue() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        dialog = VpnSettingsDialog.Mtu(mtuEditValue = EMPTY_STRING),
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText(EMPTY_STRING).assertExists()
    }

    @Test
    fun testMtuDialogWithEditValue() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState = VpnSettingsUiState.createDefault(mtu = VALID_DUMMY_MTU_VALUE),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText(VALID_DUMMY_MTU_VALUE).assertExists()
    }

    @Test
    fun testMtuDialogTextInput() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        dialog = VpnSettingsDialog.Mtu(mtuEditValue = EMPTY_STRING)
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Act
        composeTestRule.onNodeWithText(EMPTY_STRING).performTextInput(VALID_DUMMY_MTU_VALUE)

        // Assert
        composeTestRule.onNodeWithText(VALID_DUMMY_MTU_VALUE).assertExists()
    }

    @Test
    fun testMtuDialogSubmitOfValidValue() {
        // Arrange
        val mockedSubmitHandler: (Int) -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        dialog = VpnSettingsDialog.Mtu(mtuEditValue = VALID_DUMMY_MTU_VALUE)
                    ),
                onSaveMtuClick = mockedSubmitHandler,
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Act
        composeTestRule.onNodeWithText("Submit").assertIsEnabled().performClick()

        // Assert
        verify { mockedSubmitHandler.invoke(VALID_DUMMY_MTU_VALUE.toInt()) }
    }

    @Test
    fun testMtuDialogSubmitButtonDisabledWhenInvalidInput() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        dialog = VpnSettingsDialog.Mtu(mtuEditValue = INVALID_DUMMY_MTU_VALUE)
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Submit").assertIsNotEnabled()
    }

    @Test
    fun testMtuDialogResetClick() {
        // Arrange
        val mockedClickHandler: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        dialog = VpnSettingsDialog.Mtu(mtuEditValue = EMPTY_STRING)
                    ),
                onRestoreMtuClick = mockedClickHandler,
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Act
        composeTestRule.onNodeWithText("Reset to default").performClick()

        // Assert
        verify { mockedClickHandler.invoke() }
    }

    @Test
    fun testMtuDialogCancelClick() {
        // Arrange
        val mockedClickHandler: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        dialog = VpnSettingsDialog.Mtu(mtuEditValue = EMPTY_STRING)
                    ),
                onCancelMtuDialogClick = mockedClickHandler,
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Cancel").performClick()

        // Assert
        verify { mockedClickHandler.invoke() }
    }

    @Test
    fun testCustomDnsAddressesAndAddButtonVisibleWhenCustomDnsEnabled() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        isCustomDnsEnabled = true,
                        isAllowLanEnabled = false,
                        customDnsItems =
                            listOf(
                                CustomDnsItem(address = DUMMY_DNS_ADDRESS, false),
                                CustomDnsItem(address = DUMMY_DNS_ADDRESS_2, false),
                                CustomDnsItem(address = DUMMY_DNS_ADDRESS_3, false)
                            )
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }
        //        composeTestRule
        //            .onNodeWithTag(LAZY_LIST_TEST_TAG)
        //            .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))
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
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
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
                        isAllowLanEnabled = true,
                        customDnsItems =
                            listOf(CustomDnsItem(address = DUMMY_DNS_ADDRESS, isLocal = true))
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
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
                        isAllowLanEnabled = false,
                        customDnsItems =
                            listOf(CustomDnsItem(address = DUMMY_DNS_ADDRESS, isLocal = false))
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
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
                        isAllowLanEnabled = true,
                        customDnsItems =
                            listOf(CustomDnsItem(address = DUMMY_DNS_ADDRESS, isLocal = false))
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
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
                        isAllowLanEnabled = false,
                        customDnsItems =
                            listOf(CustomDnsItem(address = DUMMY_DNS_ADDRESS, isLocal = true))
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }
        composeTestRule
            .onNodeWithTag(LAZY_LIST_TEST_TAG)
            .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))

        // Assert
        composeTestRule.apply {
            onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertExists()
        }
    }

    @Test
    fun testClickAddDns() {
        // Arrange
        val mockedClickHandler: (Int?) -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState = VpnSettingsUiState.createDefault(isCustomDnsEnabled = true),
                onDnsClick = mockedClickHandler,
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }
        //        composeTestRule
        //            .onNodeWithTag(LAZY_LIST_TEST_TAG)
        //            .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))

        // Act
        composeTestRule.onNodeWithText("Add a server").performClick()

        // Assert
        verify { mockedClickHandler.invoke(null) }
    }

    @Test
    fun testShowDnsDialogForNewDnsServer() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        dialog =
                            VpnSettingsDialog.Dns(
                                stagedDns =
                                    StagedDns.NewDns(
                                        item = CustomDnsItem(DUMMY_DNS_ADDRESS, isLocal = false)
                                    ),
                            )
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Add DNS server").assertExists()
    }

    @Test
    fun testShowDnsDialogForUpdatingDnsServer() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        dialog =
                            VpnSettingsDialog.Dns(
                                stagedDns =
                                    StagedDns.EditDns(
                                        item = CustomDnsItem(DUMMY_DNS_ADDRESS, isLocal = false),
                                        index = 0
                                    )
                            )
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Update DNS server").assertExists()
    }

    @Test
    fun testDnsDialogLanWarningShownWhenLanTrafficDisabledAndLocalAddressUsed() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        dialog =
                            VpnSettingsDialog.Dns(
                                stagedDns =
                                    StagedDns.NewDns(
                                        item = CustomDnsItem(DUMMY_DNS_ADDRESS, isLocal = true),
                                        validationResult = StagedDns.ValidationResult.Success
                                    ),
                            ),
                        isAllowLanEnabled = false
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText(LOCAL_DNS_SERVER_WARNING).assertExists()
    }

    @Test
    fun testDnsDialogLanWarningNotShownWhenLanTrafficEnabledAndLocalAddressUsed() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        dialog =
                            VpnSettingsDialog.Dns(
                                stagedDns =
                                    StagedDns.NewDns(
                                        item = CustomDnsItem(DUMMY_DNS_ADDRESS, isLocal = true),
                                        validationResult = StagedDns.ValidationResult.Success
                                    ),
                            ),
                        isAllowLanEnabled = true
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
    }

    @Test
    fun testDnsDialogLanWarningNotShownWhenLanTrafficEnabledAndNonLocalAddressUsed() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        dialog =
                            VpnSettingsDialog.Dns(
                                stagedDns =
                                    StagedDns.NewDns(
                                        item = CustomDnsItem(DUMMY_DNS_ADDRESS, isLocal = false),
                                        validationResult = StagedDns.ValidationResult.Success
                                    ),
                            ),
                        isAllowLanEnabled = true
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
    }

    @Test
    fun testDnsDialogLanWarningNotShownWhenLanTrafficDisabledAndNonLocalAddressUsed() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        dialog =
                            VpnSettingsDialog.Dns(
                                stagedDns =
                                    StagedDns.NewDns(
                                        item = CustomDnsItem(DUMMY_DNS_ADDRESS, isLocal = false),
                                        validationResult = StagedDns.ValidationResult.Success
                                    ),
                            ),
                        isAllowLanEnabled = false
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
    }

    @Test
    fun testDnsDialogSubmitButtonDisabledOnInvalidDnsAddress() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        dialog =
                            VpnSettingsDialog.Dns(
                                stagedDns =
                                    StagedDns.NewDns(
                                        item = CustomDnsItem(DUMMY_DNS_ADDRESS, isLocal = false),
                                        validationResult = StagedDns.ValidationResult.InvalidAddress
                                    )
                            )
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Submit").assertIsNotEnabled()
    }

    @Test
    fun testDnsDialogSubmitButtonDisabledOnDuplicateDnsAddress() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        dialog =
                            VpnSettingsDialog.Dns(
                                stagedDns =
                                    StagedDns.NewDns(
                                        item = CustomDnsItem(DUMMY_DNS_ADDRESS, isLocal = false),
                                        validationResult =
                                            StagedDns.ValidationResult.DuplicateAddress
                                    )
                            ),
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Submit").assertIsNotEnabled()
    }

    @Test
    fun testShowSelectedTunnelQuantumOption() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(quantumResistant = QuantumResistantState.On),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
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
                onSelectQuantumResistanceSetting = mockSelectQuantumResistantSettingListener,
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
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
    fun testShowTunnelQuantumInfo() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        dialog = VpnSettingsDialog.QuantumResistanceInfo
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Got it!").assertExists()
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
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
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
                onWireguardPortSelected = mockSelectWireguardPortSelectionListener,
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
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
    fun testShowWireguardPortInfo() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        dialog =
                            VpnSettingsDialog.WireguardPortInfo(
                                availablePortRanges = listOf(PortRange(53, 53), PortRange(120, 121))
                            )
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule
            .onNodeWithText(
                "The automatic setting will randomly choose from the valid port ranges shown below."
            )
            .assertExists()
    }

    @Test
    fun testShowWireguardCustomPortDialog() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        dialog =
                            VpnSettingsDialog.CustomPort(
                                availablePortRanges = listOf(PortRange(53, 53), PortRange(120, 121))
                            )
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Valid ranges: 53, 120-121").assertExists()
    }

    @Test
    fun testShowWireguardCustomPort() {
        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        selectedWireguardPort = Constraint.Only(Port(4000))
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
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
    fun testClickWireguardCustomPortMainCell() {
        // Arrange
        val mockOnShowCustomPortDialog: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState = VpnSettingsUiState.createDefault(),
                onShowCustomPortDialog = mockOnShowCustomPortDialog,
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
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
                onShowCustomPortDialog = mockOnShowCustomPortDialog,
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
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

    @Test
    fun testSelectWireguardCustomPort() {
        // Arrange
        val onWireguardPortSelected: (Constraint<Port>) -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        selectedWireguardPort = Constraint.Only(Port(4000))
                    ),
                onWireguardPortSelected = onWireguardPortSelected,
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
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

    @Test
    fun testShowWireguardCustomPortDialogInvalidInt() {
        // Input a number to make sure that a too long number does not show and it does not crash
        // the app

        // Arrange
        composeTestRule.setContentWithTheme {
            VpnSettingsScreen(
                uiState =
                    VpnSettingsUiState.createDefault(
                        dialog =
                            VpnSettingsDialog.CustomPort(
                                availablePortRanges = listOf(PortRange(53, 53), PortRange(120, 121))
                            )
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Act
        composeTestRule
            .onNodeWithTag(CUSTOM_PORT_DIALOG_INPUT_TEST_TAG)
            .performTextInput("21474836471")

        // Assert
        composeTestRule
            .onNodeWithTagAndText(CUSTOM_PORT_DIALOG_INPUT_TEST_TAG, "21474836471")
            .assertDoesNotExist()
    }

    companion object {
        private const val LOCAL_DNS_SERVER_WARNING =
            "The local DNS server will not work unless you enable " +
                "\"Local Network Sharing\" under Preferences."
        private const val EMPTY_STRING = ""
        private const val VALID_DUMMY_MTU_VALUE = "1337"
        private const val INVALID_DUMMY_MTU_VALUE = "1111"
        private const val DUMMY_DNS_ADDRESS = "0.0.0.1"
        private const val DUMMY_DNS_ADDRESS_2 = "0.0.0.2"
        private const val DUMMY_DNS_ADDRESS_3 = "0.0.0.3"
    }
}
