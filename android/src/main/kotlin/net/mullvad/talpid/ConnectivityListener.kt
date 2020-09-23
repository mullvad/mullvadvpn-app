package net.mullvad.talpid

import android.content.Context
import android.net.ConnectivityManager
import android.net.ConnectivityManager.NetworkCallback
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import kotlin.properties.Delegates.observable
import net.mullvad.talpid.util.EventNotifier

class ConnectivityListener {
    private val availableNetworks = HashSet<Network>()

    private val callback = object : NetworkCallback() {
        override fun onAvailable(network: Network) {
            availableNetworks.add(network)
            isConnected = true
        }

        override fun onLost(network: Network) {
            availableNetworks.remove(network)
            isConnected = !availableNetworks.isEmpty()
        }
    }

    private lateinit var connectivityManager: ConnectivityManager

    val connectivityNotifier = EventNotifier(false)

    var isConnected by observable(false) { _, oldValue, newValue ->
        if (newValue != oldValue) {
            if (senderAddress != 0L) {
                notifyConnectivityChange(newValue, senderAddress)
            }

            connectivityNotifier.notify(newValue)
        }
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

    fun unregister() {
        connectivityManager.unregisterNetworkCallback(callback)
    }

    private fun finalize() {
        destroySender(senderAddress)
        senderAddress = 0L
    }

    private external fun notifyConnectivityChange(isConnected: Boolean, senderAddress: Long)
    private external fun destroySender(senderAddress: Long)
}
