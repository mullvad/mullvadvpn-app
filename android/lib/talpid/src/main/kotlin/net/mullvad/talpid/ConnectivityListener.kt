package net.mullvad.talpid

import android.net.ConnectivityManager
import android.net.LinkProperties
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import co.touchlab.kermit.Logger
import java.net.DatagramSocket
import java.net.InetAddress
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.scan
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.plus
import net.mullvad.talpid.model.ConnectionStatus
import net.mullvad.talpid.util.IPAvailabilityUtils
import net.mullvad.talpid.util.NetworkEvent
import net.mullvad.talpid.util.defaultNetworkFlow
import net.mullvad.talpid.util.networkFlow

class ConnectivityListener(
    val connectivityManager: ConnectivityManager,
    val protect: (socket: DatagramSocket) -> Unit,
) {
    private lateinit var _isConnected: StateFlow<ConnectionStatus>
    // Used by JNI
    val isConnected
        get() = _isConnected.value

    private lateinit var _currentDnsServers: StateFlow<List<InetAddress>>
    // Used by JNI
    val currentDnsServers
        get() = ArrayList(_currentDnsServers.value)

    fun register(scope: CoroutineScope) {
        _currentDnsServers =
            dnsServerChanges().stateIn(scope, SharingStarted.Eagerly, currentDnsServers())

        _isConnected =
            combine(connectivityManager.defaultNetworkFlow(), hasInternetCapability()) {
                    linkPropertiesChanged: NetworkEvent,
                    hasInternetCapability: Boolean ->
                    if (hasInternetCapability) {
                        ConnectionStatus(
                                IPAvailabilityUtils.isIPv4Available(protect = { protect(it) }),
                                IPAvailabilityUtils.isIPv6Available(protect = { protect(it) }),
                            )
                            // If we have internet, but both IPv4 and IPv6 are not available, we
                            // assume something is wrong and instead
                            // will return both as available since this is the previous behavior.
                            .takeUnless { !it.ipv4 && !it.ipv6 } ?: ConnectionStatus(true, true)
                    } else {
                        ConnectionStatus(false, false)
                    }
                }
                .onEach { notifyConnectivityChange(it.ipv4, it.ipv6) }
                .stateIn(
                    scope + Dispatchers.IO,
                    SharingStarted.Eagerly,
                    ConnectionStatus(false, false),
                )
    }

    private fun dnsServerChanges(): Flow<List<InetAddress>> =
        connectivityManager
            .defaultNetworkFlow()
            .filterIsInstance<NetworkEvent.LinkPropertiesChanged>()
            .onEach { Logger.d("Link properties changed") }
            .map { it.linkProperties.dnsServersWithoutFallback() }

    private fun currentDnsServers(): List<InetAddress> =
        connectivityManager
            .getLinkProperties(connectivityManager.activeNetwork)
            ?.dnsServersWithoutFallback() ?: emptyList()

    private fun LinkProperties.dnsServersWithoutFallback(): List<InetAddress> =
        dnsServers.filter { it.hostAddress != TalpidVpnService.FALLBACK_DUMMY_DNS_SERVER }

    private fun hasInternetCapability(): Flow<Boolean> {
        val request =
            NetworkRequest.Builder()
                .addCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
                .addCapability(NetworkCapabilities.NET_CAPABILITY_NOT_VPN)
                .build()

        return connectivityManager
            .networkFlow(request)
            .scan(setOf<Network>()) { networks, event ->
                when (event) {
                    is NetworkEvent.Available -> {
                        Logger.d("Network available ${event.network}")
                        (networks + event.network).also {
                            Logger.d("Number of networks: ${it.size}")
                        }
                    }
                    is NetworkEvent.Lost -> {
                        Logger.d("Network lost ${event.network}")
                        (networks - event.network).also {
                            Logger.d("Number of networks: ${it.size}")
                        }
                    }
                    else -> networks
                }
            }
            .map { it.isNotEmpty() }
            .distinctUntilChanged()
    }

    private external fun notifyConnectivityChange(isIPv4: Boolean, isIPv6: Boolean)
}
