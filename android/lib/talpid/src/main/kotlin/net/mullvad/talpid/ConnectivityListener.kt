package net.mullvad.talpid

import android.net.ConnectivityManager
import android.net.LinkProperties
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import co.touchlab.kermit.Logger
import java.net.DatagramSocket
import java.net.InetAddress
import kotlin.collections.ArrayList
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.scan
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.plus
import net.mullvad.talpid.model.ConnectionStatus
import net.mullvad.talpid.model.NetworkState
import net.mullvad.talpid.util.IPAvailabilityUtils
import net.mullvad.talpid.util.NetworkEvent
import net.mullvad.talpid.util.RawNetworkState
import net.mullvad.talpid.util.defaultRawNetworkStateFlow
import net.mullvad.talpid.util.networkEvents

class ConnectivityListener(
    val connectivityManager: ConnectivityManager,
    val protect: (socket: DatagramSocket) -> Unit,
) {
    private lateinit var _isConnected: StateFlow<ConnectionStatus>
    // Used by JNI
    val isConnected
        get() = _isConnected.value

    private lateinit var _currentNetworkState: StateFlow<NetworkState?>

    // Used by JNI
    val currentDefaultNetworkState: NetworkState?
        get() = _currentNetworkState.value

    // Used by JNI
    val currentDnsServers: ArrayList<InetAddress>
        get() = _currentNetworkState.value?.dnsServers ?: ArrayList()

    fun register(scope: CoroutineScope) {
        // Consider implementing retry logic for the flows below, because registering a listener on
        // the default network may fail if the network on Android 11
        // https://issuetracker.google.com/issues/175055271?pli=1
        _currentNetworkState =
            connectivityManager
                .defaultRawNetworkStateFlow()
                .map { it?.toNetworkState() }
                .onEach { notifyDefaultNetworkChange(it) }
                .stateIn(scope, SharingStarted.Eagerly, null)

        _isConnected =
            combine(_currentNetworkState, hasInternetCapability()) {
                    linkPropertiesChanged: NetworkState,
                    hasInternetCapability: Boolean ->
                    if (hasInternetCapability) {
                        ConnectionStatus(
                            IPAvailabilityUtils.isIPv4Available(protect = { protect(it) }),
                            IPAvailabilityUtils.isIPv6Available(protect = { protect(it) }),
                        )
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

    private fun LinkProperties.dnsServersWithoutFallback(): List<InetAddress> =
        dnsServers.filter { it.hostAddress != TalpidVpnService.FALLBACK_DUMMY_DNS_SERVER }

    private fun hasInternetCapability(): Flow<Boolean> {
        val request =
            NetworkRequest.Builder()
                .addCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
                .addCapability(NetworkCapabilities.NET_CAPABILITY_NOT_VPN)
                .build()

        return connectivityManager
            .networkEvents(request)
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

    private fun RawNetworkState.toNetworkState(): NetworkState =
        NetworkState(
            network.networkHandle,
            linkProperties?.routes,
            linkProperties?.dnsServersWithoutFallback(),
        )

    private external fun notifyConnectivityChange(isIPv4: Boolean, isIPv6: Boolean)

    private external fun notifyDefaultNetworkChange(networkState: NetworkState?)
}
