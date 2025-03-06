package net.mullvad.talpid.util

import android.net.ConnectivityManager
import android.net.ConnectivityManager.NetworkCallback
import android.net.LinkProperties
import android.net.Network
import android.net.NetworkCapabilities
import co.touchlab.kermit.Logger
import java.net.Inet4Address
import java.net.Inet6Address
import kotlin.time.Duration.Companion.milliseconds
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.channels.trySendBlocking
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.scan
import net.mullvad.talpid.model.Connectivity

private val CONNECTIVITY_DEBOUNCE = 300.milliseconds

fun ConnectivityManager.defaultNetworkEvents(): Flow<NetworkEvent> = callbackFlow {
    val callback =
        object : NetworkCallback() {
            override fun onLinkPropertiesChanged(network: Network, linkProperties: LinkProperties) {
                super.onLinkPropertiesChanged(network, linkProperties)
                trySendBlocking(NetworkEvent.LinkPropertiesChanged(network, linkProperties))
            }

            override fun onAvailable(network: Network) {
                super.onAvailable(network)
                trySendBlocking(NetworkEvent.Available(network))
            }

            override fun onCapabilitiesChanged(
                network: Network,
                networkCapabilities: NetworkCapabilities,
            ) {
                super.onCapabilitiesChanged(network, networkCapabilities)
                trySendBlocking(NetworkEvent.CapabilitiesChanged(network, networkCapabilities))
            }

            override fun onBlockedStatusChanged(network: Network, blocked: Boolean) {
                super.onBlockedStatusChanged(network, blocked)
                trySendBlocking(NetworkEvent.BlockedStatusChanged(network, blocked))
            }

            override fun onLosing(network: Network, maxMsToLive: Int) {
                super.onLosing(network, maxMsToLive)
                trySendBlocking(NetworkEvent.Losing(network, maxMsToLive))
            }

            override fun onLost(network: Network) {
                super.onLost(network)
                trySendBlocking(NetworkEvent.Lost(network))
            }

            override fun onUnavailable() {
                super.onUnavailable()
                trySendBlocking(NetworkEvent.Unavailable)
            }
        }
    registerDefaultNetworkCallback(callback)

    awaitClose { unregisterNetworkCallback(callback) }
}

internal fun ConnectivityManager.defaultRawNetworkStateFlow(): Flow<RawNetworkState?> =
    defaultNetworkEvents().scan(null as RawNetworkState?) { state, event -> state.reduce(event) }

private fun RawNetworkState?.reduce(event: NetworkEvent): RawNetworkState? =
    when (event) {
        is NetworkEvent.Available -> RawNetworkState(network = event.network)
        is NetworkEvent.BlockedStatusChanged -> this?.copy(blockedStatus = event.blocked)
        is NetworkEvent.CapabilitiesChanged ->
            this?.copy(networkCapabilities = event.networkCapabilities)
        is NetworkEvent.LinkPropertiesChanged -> this?.copy(linkProperties = event.linkProperties)
        is NetworkEvent.Losing -> this?.copy(maxMsToLive = event.maxMsToLive)
        is NetworkEvent.Lost -> null
        NetworkEvent.Unavailable -> null
    }

sealed interface NetworkEvent {
    data class Available(val network: Network) : NetworkEvent

    data object Unavailable : NetworkEvent

    data class LinkPropertiesChanged(val network: Network, val linkProperties: LinkProperties) :
        NetworkEvent

    data class CapabilitiesChanged(
        val network: Network,
        val networkCapabilities: NetworkCapabilities,
    ) : NetworkEvent

    data class BlockedStatusChanged(val network: Network, val blocked: Boolean) : NetworkEvent

    data class Losing(val network: Network, val maxMsToLive: Int) : NetworkEvent

    data class Lost(val network: Network) : NetworkEvent
}

data class RawNetworkState(
    val network: Network,
    val linkProperties: LinkProperties? = null,
    val networkCapabilities: NetworkCapabilities? = null,
    val blockedStatus: Boolean = false,
    val maxMsToLive: Int? = null,
)

internal fun ConnectivityManager.activeRawNetworkState(): RawNetworkState? =
    try {
        activeNetwork?.let { currentNetwork: Network ->
            RawNetworkState(
                network = currentNetwork,
                linkProperties = getLinkProperties(currentNetwork),
                networkCapabilities = getNetworkCapabilities(currentNetwork),
            )
        }
    } catch (_: RuntimeException) {
        Logger.e(
            "Unable to get active network or properties and capabilities of the active network"
        )
        null
    }

/**
 * Return a flow with the current internet connectivity status. The status is based on current
 * default network and depending on if it is a VPN. If it is not a VPN we check the network
 * properties directly and if it is a VPN we use a socket to check the underlying network. A
 * debounce is applied to avoid emitting too many events and to avoid setting the app in an offline
 * state when switching networks.
 */
@OptIn(FlowPreview::class)
fun ConnectivityManager.hasInternetConnectivity(
    resolver: UnderlyingConnectivityStatusResolver
): Flow<Connectivity.Status> =
    this.defaultRawNetworkStateFlow()
        .debounce(CONNECTIVITY_DEBOUNCE)
        .map { resolveConnectivityStatus(it, resolver) }
        .distinctUntilChanged()

internal fun resolveConnectivityStatus(
    currentRawNetworkState: RawNetworkState?,
    resolver: UnderlyingConnectivityStatusResolver,
): Connectivity.Status =
    if (currentRawNetworkState.isVpn()) {
        // If the default network is a VPN we need to use a socket to check
        // the underlying network
        resolver.currentStatus()
    } else {
        // If the default network is not a VPN we can check the addresses
        // directly
        currentRawNetworkState.toConnectivityStatus()
    }

private fun RawNetworkState?.toConnectivityStatus() =
    Connectivity.Status(
        ipv4 = this?.linkProperties?.linkAddresses?.any { it.address is Inet4Address } == true,
        ipv6 = this?.linkProperties?.linkAddresses?.any { it.address is Inet6Address } == true,
    )

private fun RawNetworkState?.isVpn(): Boolean =
    this?.networkCapabilities?.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_VPN) == false
