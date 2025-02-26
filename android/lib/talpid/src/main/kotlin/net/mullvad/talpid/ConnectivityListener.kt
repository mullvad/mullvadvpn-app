package net.mullvad.talpid

import android.net.ConnectivityManager
import android.net.LinkProperties
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import co.touchlab.kermit.Logger
import java.net.InetAddress
import kotlin.collections.ArrayList
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.scan
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
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
            hasInternetCapability()
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

    private val nonVPNNetworksRequest =
        NetworkRequest.Builder().addCapability(NetworkCapabilities.NET_CAPABILITY_NOT_VPN).build()

    private fun hasInternetCapability(): Flow<Boolean> {
        @Suppress("DEPRECATION")
        return connectivityManager
            .networkEvents(nonVPNNetworksRequest)
            .scan(
                connectivityManager.allNetworks.associateWith {
                    connectivityManager.getNetworkCapabilities(it)
                }
            ) { networks, event ->
                when (event) {
                    is NetworkEvent.Lost -> {
                        Logger.d("Network lost ${event.network}")
                        (networks - event.network).also {
                            Logger.d("Number of networks: ${it.size}")
                        }
                    }
                    is NetworkEvent.CapabilitiesChanged -> {
                        Logger.d("Network capabilities changed ${event.network}")
                        (networks + (event.network to event.networkCapabilities)).also {
                            Logger.d("Number of networks: ${it.size}")
                        }
                    }
                    else -> networks
                }
            }
            .map { it.any { it.value.hasInternetCapability() } }
            .distinctUntilChanged()
    }

    private fun NetworkCapabilities?.hasInternetCapability(): Boolean =
        this?.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) == true

    private fun RawNetworkState.toNetworkState(): NetworkState =
        NetworkState(
            network.networkHandle,
            linkProperties?.routes,
            linkProperties?.dnsServersWithoutFallback(),
        )

    private external fun notifyConnectivityChange(isConnected: Boolean)

    private external fun notifyDefaultNetworkChange(networkState: NetworkState?)
}
