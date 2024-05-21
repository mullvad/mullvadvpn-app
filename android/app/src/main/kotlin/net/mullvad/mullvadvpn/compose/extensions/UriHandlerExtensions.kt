package net.mullvad.mullvadvpn.compose.extensions

import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.UriHandler
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.common.util.createAccountUri
import net.mullvad.mullvadvpn.lib.model.WwwAuthToken

@Composable
fun UriHandler.createOpenAccountPageHook(): (WwwAuthToken) -> Unit {
    val accountUrl = stringResource(id = R.string.account_url)
    return { token -> this.openUri(createAccountUri(accountUrl, token).toString()) }
}
