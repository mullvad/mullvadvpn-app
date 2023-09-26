package net.mullvad.mullvadvpn.compose.extensions

import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.UriHandler
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.R

@Composable
fun UriHandler.createOpenAccountPageHook(): (String) -> Unit {
    val accountUrl = stringResource(id = R.string.account_url)
    return { token -> this.openUri("$accountUrl?token=$token") }
}
