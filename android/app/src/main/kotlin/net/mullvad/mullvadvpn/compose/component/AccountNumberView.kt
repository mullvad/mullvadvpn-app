package net.mullvad.mullvadvpn.compose.component

import androidx.compose.runtime.Composable
import net.mullvad.mullvadvpn.util.groupPasswordModeWithSpaces
import net.mullvad.mullvadvpn.util.groupWithSpaces

@Composable
fun AccountNumberView(accountNumber: String, shouldObfuscated: Boolean) {
    InformationView(
        content =
            if (shouldObfuscated) accountNumber.groupPasswordModeWithSpaces()
            else accountNumber.groupWithSpaces(),
        whenMissing = MissingPolicy.SHOW_SPINNER
    )
}
