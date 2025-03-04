package net.mullvad.mullvadvpn.lib.common.util

import android.content.Context
import android.content.Intent
import android.net.VpnService
import android.net.VpnService.prepare
import android.os.ParcelFileDescriptor
import arrow.core.Either
import arrow.core.flatMap
import arrow.core.left
import arrow.core.raise.either
import arrow.core.raise.ensureNotNull
import arrow.core.right
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.lib.common.util.SdkUtils.getInstalledPackagesList
import net.mullvad.mullvadvpn.lib.model.PrepareError
import net.mullvad.mullvadvpn.lib.model.Prepared

/**
 * Prepare to establish a VPN connection safely.
 *
 * Invoking VpnService.prepare() can result in 3 out comes:
 * 1. IllegalStateException - There is a legacy VPN profile marked as always on
 * 2. Intent
 *     - A: Can-prepare - Create Vpn profile or Always-on-VPN is not detected in case of Android 11+
 *     - B: Always-on-VPN - Another Vpn Profile is marked as always on (Only available up to Android
 *       11 or where testOnly is set, e.g builds from Android Studio)
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
        .flatMap { intent ->
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

/**
 * Establish a VPN connection safely.
 *
 * This function wraps the [VpnService.Builder.establish] function and catches any exceptions that
 * may be thrown and type them to a more specific error.
 *
 * @return [ParcelFileDescriptor] if successful, [EstablishError] otherwise
 */
fun VpnService.Builder.establishSafe(): Either<EstablishError, ParcelFileDescriptor> = either {
    val vpnInterfaceFd =
        Either.catch { establish() }
            .mapLeft {
                when (it) {
                    is IllegalStateException -> EstablishError.ParameterNotApplied(it)
                    is IllegalArgumentException -> EstablishError.ParameterNotAccepted(it)
                    else -> EstablishError.UnknownError(it)
                }
            }
            .bind()

    ensureNotNull(vpnInterfaceFd) { EstablishError.NullVpnInterface }

    vpnInterfaceFd
}

sealed interface EstablishError {
    data class ParameterNotApplied(val exception: IllegalStateException) : EstablishError

    data class ParameterNotAccepted(val exception: IllegalArgumentException) : EstablishError

    data object NullVpnInterface : EstablishError

    data class UnknownError(val error: Throwable) : EstablishError
}
