package net.mullvad.talpid

import android.content.Context
import android.net.ConnectivityManager
import android.net.ConnectivityManager.NetworkCallback
import android.net.LinkProperties
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import co.touchlab.kermit.Logger
import java.net.InetAddress
import kotlin.properties.Delegates.observable

class ConnectivityListener {
    private val availableNetworks = HashSet<Network>()

    private val callback =
        object : NetworkCallback() {
            override fun onAvailable(network: Network) {
                availableNetworks.add(network)
                isConnected = true
            }

            override fun onLost(network: Network) {
                availableNetworks.remove(network)
                isConnected = availableNetworks.isNotEmpty()
            }
        }

    private val defaultNetworkCallback =
        object : NetworkCallback() {
            override fun onLinkPropertiesChanged(network: Network, linkProperties: LinkProperties) {
                super.onLinkPropertiesChanged(network, linkProperties)
                currentDnsServers = ArrayList(linkProperties.dnsServers)
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

    var currentDnsServers: ArrayList<InetAddress> = ArrayList()
        private set(value) {
            field = ArrayList(value.filter { it.hostAddress != TalpidVpnService.FALLBACK_DUMMY_DNS_SERVER })
            Logger.d("New currentDnsServers: $field")
        }

    var senderAddress = 0L

    fun register(context: Context) {
        val request =
            NetworkRequest.Builder()
                .addCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
                .addCapability(NetworkCapabilities.NET_CAPABILITY_NOT_VPN)
                .build()

        connectivityManager =
            context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

        connectivityManager.registerNetworkCallback(request, callback)
        currentDnsServers =
            connectivityManager.getLinkProperties(connectivityManager.activeNetwork)?.dnsServers?.let { ArrayList(it) }
                ?: ArrayList()
        connectivityManager.registerDefaultNetworkCallback(defaultNetworkCallback)
    }

    fun unregister() {
        connectivityManager.unregisterNetworkCallback(callback)
        connectivityManager.unregisterNetworkCallback(defaultNetworkCallback)
    }

    // DROID-1401
    // This function has never been used and should most likely be merged into unregister(),
    // along with ensuring that the lifecycle of it is correct.
    @Suppress("UnusedPrivateMember")
    private fun finalize() {
        destroySender(senderAddress)
        senderAddress = 0L
    }

    private external fun notifyConnectivityChange(isConnected: Boolean, senderAddress: Long)

    private external fun destroySender(senderAddress: Long)
}
