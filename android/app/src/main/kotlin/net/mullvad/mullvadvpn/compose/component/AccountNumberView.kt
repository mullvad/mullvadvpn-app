package net.mullvad.mullvadvpn.compose.component

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import net.mullvad.mullvadvpn.lib.common.util.groupPasswordModeWithSpaces
import net.mullvad.mullvadvpn.lib.common.util.groupWithSpaces

@Composable
fun AccountNumberView(
    accountNumber: String,
    obfuscateWithPasswordDots: Boolean,
    modifier: Modifier = Modifier
) {
    InformationView(
        content =
            if (obfuscateWithPasswordDots) accountNumber.groupPasswordModeWithSpaces()
            else accountNumber.groupWithSpaces(),
        modifier = modifier,
        whenMissing = MissingPolicy.SHOW_SPINNER
    )
}
