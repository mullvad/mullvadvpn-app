package net.mullvad.mullvadvpn.compose.dialog

import android.annotation.SuppressLint
import androidx.compose.runtime.Composable
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.viewmodel.DnsDialogViewState
import org.junit.Rule
import org.junit.Test

class DnsDialogTest {
    @get:Rule val composeTestRule = createComposeRule()

    private val defaultState =
        DnsDialogViewState(
            ipAddress = "",
            validationResult = DnsDialogViewState.ValidationResult.Success,
            isLocal = false,
            isAllowLanEnabled = false,
            isNewEntry = true
        )

    @SuppressLint("ComposableNaming")
    @Composable
    private fun testDnsDialog(
        state: DnsDialogViewState = defaultState,
        onDnsInputChange: (String) -> Unit = { _ -> },
        onSaveDnsClick: () -> Unit = {},
        onRemoveDnsClick: () -> Unit = {},
        onDismiss: () -> Unit = {}
    ) {
        DnsDialog(state, onDnsInputChange, onSaveDnsClick, onRemoveDnsClick, onDismiss)
    }

    @Test
    fun testDnsDialogLanWarningShownWhenLanTrafficDisabledAndLocalAddressUsed() {
        // Arrange
        composeTestRule.setContentWithTheme {
            testDnsDialog(defaultState.copy(isAllowLanEnabled = false, isLocal = true))
        }

        // Assert
        composeTestRule.onNodeWithText(LOCAL_DNS_SERVER_WARNING).assertExists()
    }

    @Test
    fun testDnsDialogLanWarningNotShownWhenLanTrafficEnabledAndLocalAddressUsed() {
        // Arrange
        composeTestRule.setContentWithTheme {
            testDnsDialog(defaultState.copy(isAllowLanEnabled = true, isLocal = true))
        }

        // Assert
        composeTestRule.onNodeWithText(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
    }

    @Test
    fun testDnsDialogLanWarningNotShownWhenLanTrafficEnabledAndNonLocalAddressUsed() {
        // Arrange
        composeTestRule.setContentWithTheme {
            testDnsDialog(defaultState.copy(isAllowLanEnabled = true, isLocal = false))
        }

        // Assert
        composeTestRule.onNodeWithText(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
    }

    @Test
    fun testDnsDialogLanWarningNotShownWhenLanTrafficDisabledAndNonLocalAddressUsed() {
        // Arrange
        composeTestRule.setContentWithTheme {
            testDnsDialog(defaultState.copy(isAllowLanEnabled = false, isLocal = false))
        }

        // Assert
        composeTestRule.onNodeWithText(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
    }

    @Test
    fun testDnsDialogSubmitButtonDisabledOnInvalidDnsAddress() {
        // Arrange
        composeTestRule.setContentWithTheme {
            testDnsDialog(
                defaultState.copy(
                    ipAddress = invalidIpAddress,
                    validationResult = DnsDialogViewState.ValidationResult.InvalidAddress,
                )
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Submit").assertIsNotEnabled()
    }

    @Test
    fun testDnsDialogSubmitButtonDisabledOnDuplicateDnsAddress() {
        // Arrange
        composeTestRule.setContentWithTheme {
            testDnsDialog(
                defaultState.copy(
                    ipAddress = "192.168.0.1",
                    validationResult = DnsDialogViewState.ValidationResult.DuplicateAddress,
                )
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Submit").assertIsNotEnabled()
    }

    companion object {
        private const val LOCAL_DNS_SERVER_WARNING =
            "The local DNS server will not work unless you enable " +
                "\"Local Network Sharing\" under Preferences."

        private const val invalidIpAddress = "300.300.300.300"
    }
}
