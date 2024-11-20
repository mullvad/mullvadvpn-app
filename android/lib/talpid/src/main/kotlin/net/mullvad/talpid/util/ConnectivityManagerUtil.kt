package net.mullvad.talpid.util

import android.net.ConnectivityManager
import android.net.ConnectivityManager.NetworkCallback
import android.net.LinkProperties
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.channels.trySendBlocking
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow

fun ConnectivityManager.defaultNetworkFlow(): Flow<NetworkEvent> =
    callbackFlow<NetworkEvent> {
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
        registerDefaultNetworkCallback(callback)

        awaitClose { unregisterNetworkCallback(callback) }
    }

fun ConnectivityManager.networkFlow(networkRequest: NetworkRequest): Flow<NetworkEvent> =
    callbackFlow<NetworkEvent> {
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
