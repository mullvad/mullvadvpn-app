package net.mullvad.mullvadvpn.compose.component

import androidx.compose.runtime.Composable
import net.mullvad.mullvadvpn.util.groupPasswordModeWithSpaces
import net.mullvad.mullvadvpn.util.groupWithSpaces

@Composable
fun AccountNumberView(accountNumber: String, isShown: Boolean) {
    InformationView(
        content =
            if (isShown) accountNumber.groupWithSpaces()
            else accountNumber.groupPasswordModeWithSpaces(),
        whenMissing = MissingPolicy.SHOW_SPINNER
    )
}
