package net.mullvad.mullvadvpn.compose.extensions

import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.UriHandler
import androidx.compose.ui.res.stringResource
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.common.util.createAccountUri
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken

@Composable
fun UriHandler.createOpenAccountPageHook(): (WebsiteAuthToken?) -> Unit {
    val accountUrl = stringResource(id = R.string.account_url)
    return { token ->
        val accountUri = createAccountUri(accountUrl, token).toString()
        safeOpenUri(accountUri)
    }
}

fun UriHandler.createUriHook(uri: String): () -> Unit = { safeOpenUri(uri) }

private fun UriHandler.safeOpenUri(uri: String) {
    try {
        openUri(uri)
    } catch (e: IllegalArgumentException) {
        // E.g user has no browser or invalid uri
        Logger.e("Failed to open uri: $uri", e)
        e.cause?.let { Logger.e("cause:", it) }
    }
}
