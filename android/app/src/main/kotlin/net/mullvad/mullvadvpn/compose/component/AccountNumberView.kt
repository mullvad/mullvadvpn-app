package net.mullvad.mullvadvpn.compose.component

import androidx.compose.runtime.Composable
import net.mullvad.mullvadvpn.lib.common.util.groupPasswordModeWithSpaces
import net.mullvad.mullvadvpn.lib.common.util.groupWithSpaces

@Composable
fun AccountNumberView(accountNumber: String, doObfuscateWithPasswordDots: Boolean) {
    InformationView(
        content =
            if (doObfuscateWithPasswordDots) accountNumber.groupPasswordModeWithSpaces()
            else accountNumber.groupWithSpaces(),
        whenMissing = MissingPolicy.SHOW_SPINNER
    )
}
