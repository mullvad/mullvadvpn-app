package net.mullvad.talpid

import android.net.ConnectivityManager
import android.net.LinkProperties
import java.net.InetAddress
import kotlin.collections.ArrayList
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.talpid.model.NetworkState
import net.mullvad.talpid.util.RawNetworkState
import net.mullvad.talpid.util.defaultRawNetworkStateFlow
import net.mullvad.talpid.util.hasInternetConnectivity

class ConnectivityListener(private val connectivityManager: ConnectivityManager) {
    private lateinit var _isConnected: StateFlow<Boolean>
    // Used by JNI
    val isConnected
        get() = _isConnected.value

    private val _mutableNetworkState = MutableStateFlow<NetworkState?>(null)
    private val resetNetworkState: Channel<Unit> = Channel()

    // Used by JNI
    val currentDefaultNetworkState: NetworkState?
        get() = _mutableNetworkState.value

    // Used by JNI
    val currentDnsServers: ArrayList<InetAddress>
        get() = _mutableNetworkState.value?.dnsServers ?: ArrayList()

    fun register(scope: CoroutineScope) {
        // Consider implementing retry logic for the flows below, because registering a listener on
        // the default network may fail if the network on Android 11
        // https://issuetracker.google.com/issues/175055271?pli=1
        scope.launch {
            merge(
                    connectivityManager.defaultRawNetworkStateFlow(),
                    resetNetworkState.receiveAsFlow().map { null },
                )
                .map { it?.toNetworkState() }
                .onEach { notifyDefaultNetworkChange(it) }
                .collect(_mutableNetworkState)
        }

        _isConnected =
            connectivityManager
                .hasInternetConnectivity()
                .onEach { notifyConnectivityChange(it) }
                .stateIn(
                    scope,
                    SharingStarted.Eagerly,
                    true, // Assume we have internet until we know otherwise
                )
    }

    /**
     * Invalidates the network state cache. E.g when the VPN is connected or disconnected, and we
     * know the last known values not to be correct anymore.
     */
    fun invalidateNetworkStateCache() {
        _mutableNetworkState.value = null
    }

    private fun LinkProperties.dnsServersWithoutFallback(): List<InetAddress> =
        dnsServers.filter { it.hostAddress != TalpidVpnService.FALLBACK_DUMMY_DNS_SERVER }

    private fun RawNetworkState.toNetworkState(): NetworkState =
        NetworkState(
            network.networkHandle,
            linkProperties?.routes,
            linkProperties?.dnsServersWithoutFallback(),
        )

    private external fun notifyConnectivityChange(isConnected: Boolean)

    private external fun notifyDefaultNetworkChange(networkState: NetworkState?)
}
