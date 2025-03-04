package net.mullvad.talpid.util

import android.net.ConnectivityManager
import android.net.ConnectivityManager.NetworkCallback
import android.net.LinkProperties
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import co.touchlab.kermit.Logger
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.channels.trySendBlocking
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.scan
import net.mullvad.mullvadvpn.lib.common.util.debounceFirst

private val INITIAL_CONNECTIVITY_TIMEOUT = 1.seconds
private val DEFAULT_CONNECTIVITY_DEBOUNCE = 300.milliseconds

internal fun ConnectivityManager.defaultNetworkEvents(): Flow<NetworkEvent> = callbackFlow {
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

fun ConnectivityManager.networkEvents(networkRequest: NetworkRequest): Flow<NetworkEvent> =
    callbackFlow {
        val callback =
            object : NetworkCallback() {
                override fun onLinkPropertiesChanged(
                    network: Network,
                    linkProperties: LinkProperties,
                ) {
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
        registerNetworkCallback(networkRequest, callback)

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

internal data class RawNetworkState(
    val network: Network,
    val linkProperties: LinkProperties? = null,
    val networkCapabilities: NetworkCapabilities? = null,
    val blockedStatus: Boolean = false,
    val maxMsToLive: Int? = null,
)

private val nonVPNInternetNetworksRequest =
    NetworkRequest.Builder()
        .addCapability(NetworkCapabilities.NET_CAPABILITY_NOT_VPN)
        .addCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
        .build()

private sealed interface InternalConnectivityEvent {
    data class Available(val network: Network) : InternalConnectivityEvent

    data class Lost(val network: Network) : InternalConnectivityEvent
}

/**
 * Return a flow notifying us if we have internet connectivity. Initial state will be taken from
 * `allNetworks` and then updated when network events occur. Important to note that `allNetworks`
 * may return a network that we never get updates from if turned off at the moment of the initial
 * query.
 */
fun ConnectivityManager.hasInternetConnectivity(): Flow<Boolean> =
    networkEvents(nonVPNInternetNetworksRequest)
        .mapNotNull {
            when (it) {
                is NetworkEvent.Available -> InternalConnectivityEvent.Available(it.network)
                is NetworkEvent.Lost -> InternalConnectivityEvent.Lost(it.network)
                else -> null
            }
        }
        .scan(emptySet<Network>()) { networks, event ->
            when (event) {
                is InternalConnectivityEvent.Lost -> networks - event.network
                is InternalConnectivityEvent.Available -> networks + event.network
            }.also { Logger.d("Networks: $it") }
        }
        // NetworkEvents are slow, can several 100 millis to arrive. If we are online, we don't
        // want to emit a false offline with the initial accumulator, so we wait a bit before
        // emitting, and rely on `networksWithInternetConnectivity`.
        //
        // Also if our initial state was "online", but it just got turned off we might not see
        // any updates for this network even though we already were registered for updated, and
        // thus we can't drop initial value accumulator value.
        .debounceFirst(INITIAL_CONNECTIVITY_TIMEOUT, DEFAULT_CONNECTIVITY_DEBOUNCE)
        .onStart {
            // We should not use this as initial state in scan, because it may contain networks
            // that won't be included in `networkEvents` updates.
            emit(networksWithInternetConnectivity().also { Logger.d("Networks (Initial): $it") })
        }
        .map { it.isNotEmpty() }
        .distinctUntilChanged()

@Suppress("DEPRECATION")
fun ConnectivityManager.networksWithInternetConnectivity(): Set<Network> =
    // Currently the use of `allNetworks` (which is deprecated in favor of listening to network
    // events) is our only option because network events does not give us the initial state fast
    // enough.
    allNetworks
        .filter {
            val capabilities = getNetworkCapabilities(it) ?: return@filter false

            capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) &&
                capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_VPN)
        }
        .toSet()
