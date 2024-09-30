package net.mullvad.talpid

import android.content.Context
import android.net.ConnectivityManager
import android.net.ConnectivityManager.NetworkCallback
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import java.net.Inet4Address
import java.net.Inet6Address
import kotlin.properties.Delegates.observable
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.update
import net.mullvad.mullvadvpn.lib.model.NetworkInfo

class ConnectivityListener {
    private val availableNetworks = MutableStateFlow(emptySet<Network>())

    private val callback =
        object : NetworkCallback() {
            override fun onAvailable(network: Network) {
                availableNetworks.update { it + network }
                isConnected = true
            }

            override fun onLost(network: Network) {
                availableNetworks.update { it - network }
                isConnected = availableNetworks.value.isNotEmpty()
            }
        }

    private lateinit var connectivityManager: ConnectivityManager

    // Used by JNI
    var isConnected by
        observable(false) { _, oldValue, newValue ->
            if (newValue != oldValue) {
                if (senderAddress != 0L) {
                    notifyConnectivityChange(newValue, senderAddress)
                }
            }
        }

    var senderAddress = 0L

    val ipAvailability =
        availableNetworks.map { network ->
            network.map {
                val addresses =
                    connectivityManager.getLinkProperties(it)?.linkAddresses ?: emptyList()
                NetworkInfo(
                    hasIpV4 = addresses.any { it.address is Inet4Address },
                    hasIpV6 = addresses.any { it.address is Inet6Address },
                )
            }
        }

    fun register(context: Context) {
        val request =
            NetworkRequest.Builder()
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
