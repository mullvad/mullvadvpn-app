package net.mullvad.talpid

import android.net.ConnectivityManager
import android.net.LinkProperties
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import co.touchlab.kermit.Logger
import java.net.InetAddress
import kotlin.collections.ArrayList
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.scan
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.util.debounceFirst
import net.mullvad.talpid.model.NetworkState
import net.mullvad.talpid.util.NetworkEvent
import net.mullvad.talpid.util.RawNetworkState
import net.mullvad.talpid.util.defaultRawNetworkStateFlow
import net.mullvad.talpid.util.networkEvents

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
            hasInternetConnectivity()
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

    private val nonVPNInternetNetworksRequest =
        NetworkRequest.Builder()
            .addCapability(NetworkCapabilities.NET_CAPABILITY_NOT_VPN)
            .addCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
            .build()

    /**
     * Return a flow notifying us if we have internet connectivity. Initial state will be taken from
     * `allNetworks` and then updated when network events occur. Important to note that
     * `allNetworks` may return a network that we never get updates from if turned off at the moment
     * of the initial query.
     */
    private fun hasInternetConnectivity(): Flow<Boolean> {
        return connectivityManager
            .networkEvents(nonVPNInternetNetworksRequest)
            .filter { it is NetworkEvent.Lost || it is NetworkEvent.CapabilitiesChanged }
            .scan(emptySet<Network>()) { networks, event ->
                when (event) {
                    is NetworkEvent.Lost -> networks - event.network
                    is NetworkEvent.CapabilitiesChanged -> networks + event.network
                    else -> networks // Should never happen
                }.also { Logger.d("Networks: $it") }
            }
            // NetworkEvents are slow, can several 100 millis to arrive. In case we are online,
            // we don't want to emit a false offline, so we wait a bit before emitting. Also
            // if our initial state was "online" it is not given we will get events for this
            // network, and thus we can't drop initial value.
            .debounceFirst(1.seconds)
            .onStart {
                // We should not use this as initial state in scan, because it may contain networks
                // that won't be included in `networkEvents` updates.
                emit(
                    connectivityManager.networksWithInternetConnectivity().also {
                        Logger.d("Networks (Initial): $it")
                    }
                )
            }
            .map { it.isNotEmpty() }
            .distinctUntilChanged()
    }

    @Suppress("DEPRECATION")
    private fun ConnectivityManager.networksWithInternetConnectivity(): Set<Network> =
        allNetworks
            .filter {
                val capabilities = getNetworkCapabilities(it) ?: return@filter false

                capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) &&
                    capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_VPN)
            }
            .toSet()

    private fun RawNetworkState.toNetworkState(): NetworkState =
        NetworkState(
            network.networkHandle,
            linkProperties?.routes,
            linkProperties?.dnsServersWithoutFallback(),
        )

    private external fun notifyConnectivityChange(isConnected: Boolean)

    private external fun notifyDefaultNetworkChange(networkState: NetworkState?)
}
