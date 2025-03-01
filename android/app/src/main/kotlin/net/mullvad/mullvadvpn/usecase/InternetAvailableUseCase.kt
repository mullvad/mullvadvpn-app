package net.mullvad.mullvadvpn.usecase

import android.content.Context
import android.net.ConnectivityManager
import android.net.NetworkCapabilities

class InternetAvailableUseCase(val context: Context) {
    operator fun invoke(): Boolean {
        val connectivityManager =
            context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

        val network = connectivityManager.activeNetwork
        val capabilities = connectivityManager.getNetworkCapabilities(network)

        // If we are not able to fetch capabilities we should assume we have connectivity
        return capabilities?.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) ?: true
    }
}
