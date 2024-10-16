package net.mullvad.mullvadvpn.compose.dialog

import android.annotation.SuppressLint
import androidx.compose.runtime.Composable
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.onNodeWithText
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.viewmodel.DnsDialogViewState
import net.mullvad.mullvadvpn.viewmodel.ValidationError
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class DnsDialogTest {
    @OptIn(ExperimentalTestApi::class)
    @JvmField
    @RegisterExtension
    val composeExtension = createEdgeToEdgeComposeExtension()

    private val defaultState =
        DnsDialogViewState(
            input = "",
            validationError = null,
            isLocal = false,
            isAllowLanEnabled = false,
            index = null,
        )

    @SuppressLint("ComposableNaming")
    @Composable
    private fun testDnsDialog(
        state: DnsDialogViewState = defaultState,
        onDnsInputChange: (String) -> Unit = { _ -> },
        onSaveDnsClick: () -> Unit = {},
        onRemoveDnsClick: (Int) -> Unit = {},
        onDismiss: () -> Unit = {},
    ) {
        DnsDialog(state, onDnsInputChange, onSaveDnsClick, onRemoveDnsClick, onDismiss)
    }

    @Test
    fun testDnsDialogLanWarningShownWhenLanTrafficDisabledAndLocalAddressUsed() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                testDnsDialog(defaultState.copy(isAllowLanEnabled = false, isLocal = true))
            }

            // Assert
            onNodeWithText(LOCAL_DNS_SERVER_WARNING).assertExists()
        }

    @Test
    fun testDnsDialogLanWarningNotShownWhenLanTrafficEnabledAndLocalAddressUsed() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                testDnsDialog(defaultState.copy(isAllowLanEnabled = true, isLocal = true))
            }

            // Assert
            onNodeWithText(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
        }

    @Test
    fun testDnsDialogLanWarningNotShownWhenLanTrafficEnabledAndNonLocalAddressUsed() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                testDnsDialog(defaultState.copy(isAllowLanEnabled = true, isLocal = false))
            }

            // Assert
            onNodeWithText(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
        }

    @Test
    fun testDnsDialogLanWarningNotShownWhenLanTrafficDisabledAndNonLocalAddressUsed() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                testDnsDialog(defaultState.copy(isAllowLanEnabled = false, isLocal = false))
            }

            // Assert
            onNodeWithText(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
        }

    @Test
    fun testDnsDialogSubmitButtonDisabledOnInvalidDnsAddress() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                testDnsDialog(
                    defaultState.copy(
                        input = invalidIpAddress,
                        validationError = ValidationError.InvalidAddress,
                    )
                )
            }

            // Assert
            onNodeWithText("Submit").assertIsNotEnabled()
        }

    @Test
    fun testDnsDialogSubmitButtonDisabledOnDuplicateDnsAddress() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                testDnsDialog(
                    defaultState.copy(
                        input = "192.168.0.1",
                        validationError = ValidationError.DuplicateAddress,
                    )
                )
            }

            // Assert
            onNodeWithText("Submit").assertIsNotEnabled()
        }

    companion object {
        private const val LOCAL_DNS_SERVER_WARNING =
            "The local DNS server will not work unless you enable " +
                "\"Local Network Sharing\" under VPN settings."

        private const val invalidIpAddress = "300.300.300.300"
    }
}
