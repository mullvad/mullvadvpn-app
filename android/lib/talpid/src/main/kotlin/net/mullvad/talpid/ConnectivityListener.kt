package net.mullvad.talpid

import android.content.Context
import android.net.ConnectivityManager
import android.net.ConnectivityManager.NetworkCallback
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import co.touchlab.kermit.Logger
import java.net.Inet4Address
import java.net.Inet6Address
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import net.mullvad.talpid.model.NetworkInfo

class ConnectivityListener {
    private val availableNetworks = MutableStateFlow(emptySet<Network>())

    private val callback =
        object : NetworkCallback() {
            override fun onAvailable(network: Network) {
                availableNetworks.update { it + network }
                val info = availableNetworks.value.info()
                notifyConnectivityChange(info.hasIpV4, info.hasIpV6, senderAddress)
            }

            override fun onLost(network: Network) {
                availableNetworks.update { it - network }
                val info = availableNetworks.value.info()
                notifyConnectivityChange(info.hasIpV4, info.hasIpV6, senderAddress)
            }
        }

    private lateinit var connectivityManager: ConnectivityManager

    var senderAddress = 0L

    // Used by jni
    @Suppress("unused") fun isConnected(): Boolean = availableNetworks.value.info().isConnected

    fun Set<Network>.info(): NetworkInfo {
        return this.map { network ->
                val addresses =
                    connectivityManager.getLinkProperties(network)?.linkAddresses ?: emptyList()
                NetworkInfo(
                    hasIpV4 = addresses.any { it.address is Inet4Address },
                    hasIpV6 = addresses.any { it.address is Inet6Address },
                )
            }
            .reduceOrNull { acc, networkInfo ->
                NetworkInfo(
                    hasIpV4 = acc.hasIpV4 || networkInfo.hasIpV4,
                    hasIpV6 = acc.hasIpV6 || networkInfo.hasIpV6,
                )
            } ?: NetworkInfo(hasIpV4 = false, hasIpV6 = false)
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

    // DROID-1401
    // This function has never been used and should most likely be merged into unregister(),
    // along with ensuring that the lifecycle of it is correct.
    @Suppress("UnusedPrivateMember")
    private fun finalize() {
        destroySender(senderAddress)
        senderAddress = 0L
    }

    private external fun notifyConnectivityChange(ipv4: Boolean, ipv6: Boolean, senderAddress: Long)

    private external fun destroySender(senderAddress: Long)
}
