package net.mullvad.mullvadvpn.lib.usecase

import android.Manifest
import android.content.Context
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import androidx.annotation.RequiresPermission

/**
 * Checks for internet availability on the device.
 *
 * NOTE! This check is unreliable and should not be used to gate network requests, only to check for
 * issues after a network request has failed.
 */
class InternetAvailableUseCase(val context: Context) {
    @RequiresPermission(Manifest.permission.ACCESS_NETWORK_STATE)
    operator fun invoke(): Boolean {
        val connectivityManager =
            context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

        val network = connectivityManager.activeNetwork
        val capabilities = connectivityManager.getNetworkCapabilities(network)

        return capabilities?.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) ?: false
    }
}
