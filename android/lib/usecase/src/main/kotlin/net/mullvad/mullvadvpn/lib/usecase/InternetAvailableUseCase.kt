package net.mullvad.mullvadvpn.lib.usecase

import android.content.Context
import android.net.ConnectivityManager
import android.net.NetworkCapabilities

/**
 * Checks for internet availability on the device.
 *
 * NOTE! This check is unreliable and should not be used to gate network requests, only to check for
 * issues after a network request has failed.
 */
@Suppress("MissingPermission")
class InternetAvailableUseCase(val context: Context) {
    operator fun invoke(): Boolean {
        val connectivityManager =
            context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

        val network = connectivityManager.activeNetwork
        val capabilities = connectivityManager.getNetworkCapabilities(network)

        return capabilities?.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) ?: false
    }
}
