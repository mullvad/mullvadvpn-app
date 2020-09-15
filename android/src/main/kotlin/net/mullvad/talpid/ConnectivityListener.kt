package net.mullvad.talpid

import android.content.Context
import android.net.ConnectivityManager
import android.net.ConnectivityManager.NetworkCallback
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import net.mullvad.talpid.util.EventNotifier

class ConnectivityListener {
    private val availableNetworks = HashSet<Network>()

    private val callback = object : NetworkCallback() {
        override fun onAvailable(network: Network) {
            availableNetworks.add(network)

            if (!isConnected) {
                isConnected = true
            }
        }

        override fun onLost(network: Network) {
            availableNetworks.remove(network)

            if (isConnected && availableNetworks.isEmpty()) {
                isConnected = false
            }
        }
    }

    private lateinit var connectivityManager: ConnectivityManager

    val connectivityNotifier = EventNotifier(false)

    var isConnected = false
        private set(value) {
            field = value

            if (senderAddress != 0L) {
                notifyConnectivityChange(value, senderAddress)
            }

            connectivityNotifier.notify(value)
        }

    var senderAddress = 0L

    fun register(context: Context) {
        val request = NetworkRequest.Builder()
            .addCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
            .addCapability(NetworkCapabilities.NET_CAPABILITY_NOT_VPN)
            .build()

        connectivityManager =
            context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

        connectivityManager.registerNetworkCallback(request, callback)
    }

    fun unregister(context: Context) {
        connectivityManager.unregisterNetworkCallback(callback)
    }

    private fun finalize() {
        destroySender(senderAddress)
        senderAddress = 0L
    }

    private external fun notifyConnectivityChange(isConnected: Boolean, senderAddress: Long)
    private external fun destroySender(senderAddress: Long)
}
