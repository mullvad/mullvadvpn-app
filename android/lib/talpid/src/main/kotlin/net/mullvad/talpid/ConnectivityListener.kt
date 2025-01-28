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
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.scan
import kotlinx.coroutines.flow.stateIn
import net.mullvad.talpid.model.NetworkState as DaemonNetworkState
import net.mullvad.talpid.util.NetworkEvent
import net.mullvad.talpid.util.NetworkState
import net.mullvad.talpid.util.defaultNetworkStateFlow
import net.mullvad.talpid.util.networkFlow

class ConnectivityListener(val connectivityManager: ConnectivityManager) {
    private lateinit var _isConnected: StateFlow<Boolean>
    // Used by JNI
    val isConnected
        get() = _isConnected.value

    private lateinit var _currentNetworkState: StateFlow<DaemonNetworkState?>

    val currentDefaultNetworkState: DaemonNetworkState?
        get() = _currentNetworkState.value

    // Used by JNI
    val currentDnsServers: ArrayList<InetAddress>
        get() = _currentNetworkState.value?.dnsServers ?: ArrayList()

    fun register(scope: CoroutineScope) {
        _currentNetworkState =
            connectivityManager
                .defaultNetworkStateFlow()
                .map { it?.toDaemonNetworkState() }
                .onEach { notifyDefaultNetworkChange(it) }
                .stateIn(scope, SharingStarted.Eagerly, null)

        _isConnected =
            hasInternetCapability()
                .onEach { notifyConnectivityChange(it) }
                .stateIn(scope, SharingStarted.Eagerly, false)
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

    private fun NetworkState.toDaemonNetworkState(): DaemonNetworkState =
        DaemonNetworkState(
            network.networkHandle,
            linkProperties?.routes,
            linkProperties?.dnsServersWithoutFallback(),
        )

    private external fun notifyConnectivityChange(isConnected: Boolean)

    private external fun notifyDefaultNetworkChange(networkState: DaemonNetworkState?)
}
