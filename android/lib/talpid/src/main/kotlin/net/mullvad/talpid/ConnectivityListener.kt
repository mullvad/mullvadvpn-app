package net.mullvad.talpid

import android.net.ConnectivityManager
import android.net.LinkProperties
import android.net.Network
import android.net.NetworkCapabilities
import co.touchlab.kermit.Logger
import java.net.DatagramSocket
import java.net.Inet4Address
import java.net.Inet6Address
import java.net.InetAddress
import kotlin.collections.ArrayList
import kotlin.time.Duration.Companion.milliseconds
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.plus
import kotlinx.coroutines.runBlocking
import net.mullvad.talpid.model.Connectivity
import net.mullvad.talpid.model.NetworkState
import net.mullvad.talpid.util.IpUtils
import net.mullvad.talpid.util.RawNetworkState
import net.mullvad.talpid.util.defaultRawNetworkStateFlow

class ConnectivityListener(
    private val connectivityManager: ConnectivityManager,
    val protect: (socket: DatagramSocket) -> Boolean,
) {
    private lateinit var _isConnected: StateFlow<Connectivity>
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

    @OptIn(FlowPreview::class)
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
                .defaultRawNetworkStateFlow()
                .debounce(300.milliseconds)
                .map { it.toConnectivity() }
                .distinctUntilChanged()
                .onEach { notifyConnectivityChange(it.ipv4, it.ipv6) }
                .stateIn(
                    scope + Dispatchers.IO,
                    SharingStarted.Eagerly,
                    // Has to happen on IO to avoid NetworkOnMainThreadException, we actually don't
                    // send any traffic just open a socket to detect the IP version.
                    runBlocking(Dispatchers.IO) {
                        connectivityManager.activeRawNetworkState().toConnectivity()
                    },
                )
    }

    private fun ConnectivityManager.activeRawNetworkState(): RawNetworkState? =
        try {
            activeNetwork?.let { initialNetwork: Network ->
                RawNetworkState(
                    network = initialNetwork,
                    linkProperties = getLinkProperties(initialNetwork),
                    networkCapabilities = getNetworkCapabilities(initialNetwork),
                )
            }
        } catch (_: RuntimeException) {
            Logger.e(
                "Unable to get active network or properties and capabilities of the active network"
            )
            null
        }

    private fun RawNetworkState?.toConnectivity(): Connectivity.Status =
        if (isVpn()) {
            // If the default network is a VPN we need to use a socket to check
            // the underlying network
            Connectivity.Status(
                IpUtils.hasIPv4(protect = { protect(it) }),
                IpUtils.hasIPv6(protect = { protect(it) }),
            )
        } else {
            // If the default network is not a VPN we can check the addresses
            // directly
            Connectivity.Status(
                ipv4 =
                    this?.linkProperties?.routes?.any { it.destination.address is Inet4Address } ==
                        true,
                ipv6 =
                    this?.linkProperties?.routes?.any { it.destination.address is Inet6Address } ==
                        true,
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

    private fun RawNetworkState?.isVpn(): Boolean =
        this?.networkCapabilities?.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_VPN) ==
            false

    private fun RawNetworkState.toNetworkState(): NetworkState =
        NetworkState(
            network.networkHandle,
            linkProperties?.routes,
            linkProperties?.dnsServersWithoutFallback(),
        )

    private external fun notifyConnectivityChange(isIPv4: Boolean, isIPv6: Boolean)

    private external fun notifyDefaultNetworkChange(networkState: NetworkState?)
}
