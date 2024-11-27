package net.mullvad.mullvadvpn.lib.common.util

import android.content.Context
import android.content.Intent
import android.net.VpnService.prepare
import arrow.core.Either
import arrow.core.flatten
import arrow.core.left
import arrow.core.right
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.lib.common.util.SdkUtils.getInstalledPackagesList
import net.mullvad.mullvadvpn.lib.model.PrepareError
import net.mullvad.mullvadvpn.lib.model.Prepared

/**
 * Invoking VpnService.prepare() can result in 3 out comes:
 * 1. IllegalStateException - There is a legacy VPN profile marked as always on
 * 2. Intent
 *     - A: Can-prepare - Create Vpn profile
 *     - B: Always-on-VPN - Another Vpn Profile is marked as always on
 * 3. null - The app has the VPN permission
 *
 * In case 1 and 2b, you don't know if you have a VPN profile or not.
 */
fun Context.prepareVpnSafe(): Either<PrepareError, Prepared> =
    Either.catch {
            val intent: Intent? = prepare(this)
            intent
        }
        .mapLeft {
            Logger.e("VpnService.prepare() failed: $it")
            when (it) {
                is IllegalStateException -> PrepareError.OtherLegacyAlwaysOnVpn
                else -> throw it
            }
        }
        .map { intent ->
            if (intent == null) {
                Prepared.right()
            } else {
                val alwaysOnVpnApp = getAlwaysOnVpnAppName()
                if (alwaysOnVpnApp == null) {
                    PrepareError.NotPrepared(intent).left()
                } else {
                    PrepareError.OtherAlwaysOnApp(alwaysOnVpnApp).left()
                }
            }
        }
        .flatten()

fun Context.getAlwaysOnVpnAppName(): String? {
    return resolveAlwaysOnVpnPackageName()
        ?.let { currentAlwaysOnVpn ->
            packageManager.getInstalledPackagesList(0).singleOrNull {
                it.packageName == currentAlwaysOnVpn && it.packageName != packageName
            }
        }
        ?.applicationInfo
        ?.loadLabel(packageManager)
        ?.toString()
}
