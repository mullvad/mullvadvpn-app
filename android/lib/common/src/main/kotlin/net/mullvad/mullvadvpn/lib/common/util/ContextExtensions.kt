package net.mullvad.mullvadvpn.lib.common.util

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.core.net.toUri
import arrow.core.Either
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.lib.model.StartActivityError
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken

fun createAccountUri(accountUri: String, websiteAuthToken: WebsiteAuthToken?): Uri {
    val urlString = buildString {
        append(accountUri)
        if (websiteAuthToken != null) {
            append("?token=")
            append(websiteAuthToken.value)
        }
    }
    return urlString.toUri()
}

// Activity not found can be return if the device does not have system vpn settings available.
// This is the case for Android TV devices. In normal cases, this action should not be available
// for those devices (see SystemVpnSettingsAvailableUseCase). This is an extra safety check.

fun Context.openVpnSettings(): Either<StartActivityError.ActivityNotFound, Unit> =
    Either.catch {
            val intent = Intent("android.settings.VPN_SETTINGS")
            startActivity(intent)
        }
        .onLeft { Logger.e("Failed to open VPN settings", it) }
        .mapLeft {
            if (it is ActivityNotFoundException) {
                StartActivityError.ActivityNotFound
            }
            throw it
        }
