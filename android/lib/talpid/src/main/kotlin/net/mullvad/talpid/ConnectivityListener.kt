package net.mullvad.talpid

import android.net.ConnectivityManager
import android.net.LinkProperties
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import co.touchlab.kermit.Logger
import java.net.InetAddress
import kotlin.collections.ArrayList
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.drop
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.scan
import kotlinx.coroutines.flow.stateIn
import net.mullvad.talpid.model.NetworkState
import net.mullvad.talpid.util.NetworkEvent
import net.mullvad.talpid.util.RawNetworkState
import net.mullvad.talpid.util.defaultRawNetworkStateFlow
import net.mullvad.talpid.util.networkEvents

class ConnectivityListener(
    private val connectivityManager: ConnectivityManager,
    private val resetDnsFlow: Flow<Unit>,
) {
    private lateinit var _isConnected: StateFlow<Boolean>
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
            merge(connectivityManager.defaultRawNetworkStateFlow(), resetDnsFlow.map { null })
                .map { it?.toNetworkState() }
                .onEach {
                    Logger.d("NetworkState routes: ${it?.routes}")
                    notifyDefaultNetworkChange(it)
                }
                .stateIn(scope, SharingStarted.Eagerly, null)

        @Suppress("DEPRECATION")
        _isConnected =
            hasInternetCapability()
                .onEach { notifyConnectivityChange(it) }
                .stateIn(
                    scope,
                    SharingStarted.Eagerly,
                    true, // Assume we have internet until we know otherwise
                )
    }

    private fun LinkProperties.dnsServersWithoutFallback(): List<InetAddress> =
        dnsServers.filter { it.hostAddress != TalpidVpnService.FALLBACK_DUMMY_DNS_SERVER }

    private val nonVPNNetworksRequest =
        NetworkRequest.Builder().addCapability(NetworkCapabilities.NET_CAPABILITY_NOT_VPN).build()

    private fun hasInternetCapability(): Flow<Boolean> {

        return connectivityManager
            .networkEvents(nonVPNNetworksRequest)
            .scan(mapOf<Network, NetworkCapabilities?>()) { networks, event ->
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
            .distinctUntilChanged()
            .drop(1)
            .map { it.any { it.value?.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) == true} }
            .onEach { Logger.d("Do we have connectivity? $it") }
    }

    private fun RawNetworkState.toNetworkState(): NetworkState =
        NetworkState(
            network.networkHandle,
            linkProperties?.routes,
            linkProperties?.dnsServersWithoutFallback(),
        )

    private external fun notifyConnectivityChange(isConnected: Boolean)

    private external fun notifyDefaultNetworkChange(networkState: NetworkState?)
}
