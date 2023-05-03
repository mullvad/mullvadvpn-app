package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.material.ExperimentalMaterialApi
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import io.mockk.verifyAll
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import net.mullvad.mullvadvpn.compose.state.AdvancedSettingsUiState
import net.mullvad.mullvadvpn.viewmodel.CustomDnsItem
import net.mullvad.mullvadvpn.viewmodel.StagedDns
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class AdvancedSettingsScreenTest {
    @get:Rule val composeTestRule = createComposeRule()

    @Before
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testDefaultState() {
        // Arrange
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState = AdvancedSettingsUiState.DefaultUiState(),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.apply {
            onNodeWithText("WireGuard MTU").assertExists()
            onNodeWithText("Default").assertExists()
            onNodeWithText("Split tunneling").assertExists()
            onNodeWithText("Use custom DNS server").assertExists()
            onNodeWithText("Add a server").assertDoesNotExist()
        }
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testMtuCustomValue() {
        // Arrange
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState = AdvancedSettingsUiState.DefaultUiState(mtu = VALID_DUMMY_MTU_VALUE),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText(VALID_DUMMY_MTU_VALUE).assertExists()
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testMtuClick() {
        // Arrange
        val mockedClickHandler: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState = AdvancedSettingsUiState.DefaultUiState(),
                onMtuCellClick = mockedClickHandler,
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Act
        composeTestRule.onNodeWithText("WireGuard MTU").performClick()

        // Assert
        verify { mockedClickHandler.invoke() }
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testMtuDialogWithDefaultValue() {
        // Arrange
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState = AdvancedSettingsUiState.MtuDialogUiState(mtuEditValue = EMPTY_STRING),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText(EMPTY_STRING).assertExists()
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testMtuDialogWithEditValue() {
        // Arrange
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState =
                    AdvancedSettingsUiState.MtuDialogUiState(mtuEditValue = VALID_DUMMY_MTU_VALUE),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText(VALID_DUMMY_MTU_VALUE).assertExists()
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testMtuDialogTextInput() {
        // Arrange
        val mockedInputHandler: (String) -> Unit = mockk(relaxed = true)
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState = AdvancedSettingsUiState.MtuDialogUiState(mtuEditValue = EMPTY_STRING),
                onMtuInputChange = mockedInputHandler,
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Act
        composeTestRule.onNodeWithText(EMPTY_STRING).performTextInput(VALID_DUMMY_MTU_VALUE)

        // Assert
        verifyAll { mockedInputHandler.invoke(VALID_DUMMY_MTU_VALUE) }
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testMtuDialogSubmitOfValidValue() {
        // Arrange
        val mockedSubmitHandler: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState =
                    AdvancedSettingsUiState.MtuDialogUiState(mtuEditValue = VALID_DUMMY_MTU_VALUE),
                onSaveMtuClick = mockedSubmitHandler,
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Act
        composeTestRule.onNodeWithText("Submit").assertIsEnabled().performClick()

        // Assert
        verify { mockedSubmitHandler.invoke() }
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testMtuDialogSubmitButtonDisabledWhenInvalidInput() {
        // Arrange
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState =
                    AdvancedSettingsUiState.MtuDialogUiState(
                        mtuEditValue = INVALID_DUMMY_MTU_VALUE
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Submit").assertIsNotEnabled()
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testMtuDialogResetClick() {
        // Arrange
        val mockedClickHandler: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState = AdvancedSettingsUiState.MtuDialogUiState(mtuEditValue = EMPTY_STRING),
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
    @OptIn(ExperimentalMaterialApi::class)
    fun testMtuDialogCancelClick() {
        // Arrange
        val mockedClickHandler: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState = AdvancedSettingsUiState.MtuDialogUiState(mtuEditValue = EMPTY_STRING),
                onCancelMtuDialogClicked = mockedClickHandler,
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Cancel").performClick()

        // Assert
        verify { mockedClickHandler.invoke() }
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testClickSplitTunneling() {
        // Arrange
        val mockedClickHandler: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState = AdvancedSettingsUiState.DefaultUiState(),
                onSplitTunnelingNavigationClick = mockedClickHandler,
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Act
        composeTestRule.onNodeWithText("Split tunneling").performClick()

        // Assert
        verify { mockedClickHandler.invoke() }
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testCustomDnsAddressesAndAddButtonVisibleWhenCustomDnsEnabled() {
        // Arrange
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState =
                    AdvancedSettingsUiState.DefaultUiState(
                        isCustomDnsEnabled = true,
                        isAllowLanEnabled = false,
                        customDnsItems =
                            listOf(
                                CustomDnsItem(address = DUMMY_DNS_ADDRESS, false),
                                CustomDnsItem(address = DUMMY_DNS_ADDRESS_2, false),
                                CustomDnsItem(address = DUMMY_DNS_ADDRESS_3, false),
                            ),
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
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
    @OptIn(ExperimentalMaterialApi::class)
    fun testCustomDnsAddressesAndAddButtonNotVisibleWhenCustomDnsDisabled() {
        // Arrange
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState =
                    AdvancedSettingsUiState.DefaultUiState(
                        isCustomDnsEnabled = false,
                        customDnsItems = listOf(CustomDnsItem(address = DUMMY_DNS_ADDRESS, false)),
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText(DUMMY_DNS_ADDRESS).assertDoesNotExist()
        composeTestRule.onNodeWithText("Add a server").assertDoesNotExist()
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testLanWarningNotShownWhenLanTrafficEnabledAndLocalAddressIsUsed() {
        // Arrange
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState =
                    AdvancedSettingsUiState.DefaultUiState(
                        isCustomDnsEnabled = true,
                        isAllowLanEnabled = true,
                        customDnsItems =
                            listOf(CustomDnsItem(address = DUMMY_DNS_ADDRESS, isLocal = true)),
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testLanWarningNotShowedWhenLanTrafficDisabledAndLocalAddressIsNotUsed() {
        // Arrange
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState =
                    AdvancedSettingsUiState.DefaultUiState(
                        isCustomDnsEnabled = true,
                        isAllowLanEnabled = false,
                        customDnsItems =
                            listOf(CustomDnsItem(address = DUMMY_DNS_ADDRESS, isLocal = false)),
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testLanWarningNotShowedWhenLanTrafficEnabledAndLocalAddressIsNotUsed() {
        // Arrange
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState =
                    AdvancedSettingsUiState.DefaultUiState(
                        isCustomDnsEnabled = true,
                        isAllowLanEnabled = true,
                        customDnsItems =
                            listOf(CustomDnsItem(address = DUMMY_DNS_ADDRESS, isLocal = false)),
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testLanWarningShowedWhenAllowLanEnabledAndLocalDnsAddressIsUsed() {
        // Arrange
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState =
                    AdvancedSettingsUiState.DefaultUiState(
                        isCustomDnsEnabled = true,
                        isAllowLanEnabled = false,
                        customDnsItems =
                            listOf(CustomDnsItem(address = DUMMY_DNS_ADDRESS, isLocal = true)),
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.apply {
            onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertExists()
        }
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testClickAddDns() {
        // Arrange
        val mockedClickHandler: (Int?) -> Unit = mockk(relaxed = true)
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState = AdvancedSettingsUiState.DefaultUiState(isCustomDnsEnabled = true),
                onDnsClick = mockedClickHandler,
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Act
        composeTestRule.onNodeWithText("Add a server").performClick()

        // Assert
        verify { mockedClickHandler.invoke(null) }
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testShowDnsDialogForNewDnsServer() {
        // Arrange
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState =
                    AdvancedSettingsUiState.DnsDialogUiState(
                        stagedDns =
                            StagedDns.NewDns(
                                item = CustomDnsItem(DUMMY_DNS_ADDRESS, isLocal = false),
                            ),
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Add DNS server").assertExists()
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testShowDnsDialogForUpdatingDnsServer() {
        // Arrange
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState =
                    AdvancedSettingsUiState.DnsDialogUiState(
                        stagedDns =
                            StagedDns.EditDns(
                                item = CustomDnsItem(DUMMY_DNS_ADDRESS, isLocal = false),
                                index = 0,
                            ),
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Update DNS server").assertExists()
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testDnsDialogLanWarningShownWhenLanTrafficDisabledAndLocalAddressUsed() {
        // Arrange
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState =
                    AdvancedSettingsUiState.DnsDialogUiState(
                        stagedDns =
                            StagedDns.NewDns(
                                item = CustomDnsItem(DUMMY_DNS_ADDRESS, isLocal = true),
                                validationResult = StagedDns.ValidationResult.Success,
                            ),
                        isAllowLanEnabled = false,
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText(LOCAL_DNS_SERVER_WARNING).assertExists()
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testDnsDialogLanWarningNotShownWhenLanTrafficEnabledAndLocalAddressUsed() {
        // Arrange
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState =
                    AdvancedSettingsUiState.DnsDialogUiState(
                        stagedDns =
                            StagedDns.NewDns(
                                item = CustomDnsItem(DUMMY_DNS_ADDRESS, isLocal = true),
                                validationResult = StagedDns.ValidationResult.Success,
                            ),
                        isAllowLanEnabled = true,
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testDnsDialogLanWarningNotShownWhenLanTrafficEnabledAndNonLocalAddressUsed() {
        // Arrange
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState =
                    AdvancedSettingsUiState.DnsDialogUiState(
                        stagedDns =
                            StagedDns.NewDns(
                                item = CustomDnsItem(DUMMY_DNS_ADDRESS, isLocal = false),
                                validationResult = StagedDns.ValidationResult.Success,
                            ),
                        isAllowLanEnabled = true,
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testDnsDialogLanWarningNotShownWhenLanTrafficDisabledAndNonLocalAddressUsed() {
        // Arrange
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState =
                    AdvancedSettingsUiState.DnsDialogUiState(
                        stagedDns =
                            StagedDns.NewDns(
                                item = CustomDnsItem(DUMMY_DNS_ADDRESS, isLocal = false),
                                validationResult = StagedDns.ValidationResult.Success,
                            ),
                        isAllowLanEnabled = false,
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testDnsDialogSubmitButtonDisabledOnInvalidDnsAddress() {
        // Arrange
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState =
                    AdvancedSettingsUiState.DnsDialogUiState(
                        stagedDns =
                            StagedDns.NewDns(
                                item = CustomDnsItem(DUMMY_DNS_ADDRESS, isLocal = false),
                                validationResult = StagedDns.ValidationResult.InvalidAddress,
                            ),
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Submit").assertIsNotEnabled()
    }

    @Test
    @OptIn(ExperimentalMaterialApi::class)
    fun testDnsDialogSubmitButtonDisabledOnDuplicateDnsAddress() {
        // Arrange
        composeTestRule.setContent {
            AdvancedSettingScreen(
                uiState =
                    AdvancedSettingsUiState.DnsDialogUiState(
                        stagedDns =
                            StagedDns.NewDns(
                                item = CustomDnsItem(DUMMY_DNS_ADDRESS, isLocal = false),
                                validationResult = StagedDns.ValidationResult.DuplicateAddress,
                            ),
                    ),
                toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Submit").assertIsNotEnabled()
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
