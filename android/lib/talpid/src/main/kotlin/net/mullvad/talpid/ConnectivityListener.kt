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
import kotlinx.coroutines.flow.reduce
import kotlinx.coroutines.flow.update
import net.mullvad.mullvadvpn.lib.model.NetworkInfo

class ConnectivityListener {
    private val availableNetworks = MutableStateFlow(emptySet<Network>())

    private val callback =
        object : NetworkCallback() {
            override fun onAvailable(network: Network) {
                availableNetworks.update { it + network }
                val tmp = availableNetworks.value.info().gather()
                notifyConnectivityChange(tmp.hasIpV4, tmp.hasIpV6, senderAddress)
            }

            override fun onLost(network: Network) {
                availableNetworks.update { it - network }
                val tmp = availableNetworks.value.info().gather()
                notifyConnectivityChange(tmp.hasIpV4, tmp.hasIpV6, senderAddress)
            }
        }

    private lateinit var connectivityManager: ConnectivityManager

    var senderAddress = 0L
    fun isConnected(): Boolean { return availableNetworks.value.info().gather().hasIpV4 || availableNetworks.value.info().gather().hasIpV6 }

    fun Set<Network>.info(): List<NetworkInfo> {
        return this.map {
            val addresses =
                connectivityManager.getLinkProperties(it)?.linkAddresses ?: emptyList()
            NetworkInfo(
                hasIpV4 = addresses.any { it.address is Inet4Address },
                hasIpV6 = addresses.any { it.address is Inet6Address },
            )
        }
    }

    fun List<NetworkInfo>.gather(): NetworkInfo {
        return this.fold(NetworkInfo(false, false)) { acc, next ->
            val accc = acc.copy()
            if (next.hasIpV4) {
                accc.hasIpV4 = true
            }
            if (next.hasIpV6) {
                accc.hasIpV6 = true
            }
            return acc
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

    private external fun notifyConnectivityChange(ipv4: Boolean, ipv6: Boolean, senderAddress: Long)

    private external fun destroySender(senderAddress: Long)
}
