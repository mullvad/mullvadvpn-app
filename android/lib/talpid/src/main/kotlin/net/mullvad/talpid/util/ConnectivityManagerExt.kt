package net.mullvad.talpid.util

import android.net.ConnectivityManager
import android.net.ConnectivityManager.NetworkCallback
import android.net.LinkProperties
import android.net.Network
import android.net.NetworkCapabilities
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.channels.trySendBlocking
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow

sealed interface NetworkEvent {
    data class OnAvailable(val network: Network) : NetworkEvent

    data object OnUnavailable : NetworkEvent

    data class OnLinkPropertiesChanged(val network: Network, val linkProperties: LinkProperties) :
        NetworkEvent

    data class OnCapabilitiesChanged(
        val network: Network,
        val networkCapabilities: NetworkCapabilities,
    ) : NetworkEvent

    data class OnBlockedStatusChanged(val network: Network, val blocked: Boolean) : NetworkEvent

    data class OnLosing(val network: Network, val maxMsToLive: Int) : NetworkEvent

    data class OnLost(val network: Network) : NetworkEvent
}

fun ConnectivityManager.defaultCallbackFlow(): Flow<NetworkEvent> =
    callbackFlow<NetworkEvent> {
        val callback =
            object : NetworkCallback() {
                override fun onLinkPropertiesChanged(
                    network: Network,
                    linkProperties: LinkProperties,
                ) {
                    super.onLinkPropertiesChanged(network, linkProperties)
                    trySendBlocking(NetworkEvent.OnLinkPropertiesChanged(network, linkProperties))
                }

                override fun onAvailable(network: Network) {
                    super.onAvailable(network)
                    trySendBlocking(NetworkEvent.OnAvailable(network))
                }

                override fun onCapabilitiesChanged(
                    network: Network,
                    networkCapabilities: NetworkCapabilities,
                ) {
                    super.onCapabilitiesChanged(network, networkCapabilities)
                    trySendBlocking(
                        NetworkEvent.OnCapabilitiesChanged(network, networkCapabilities)
                    )
                }

                override fun onBlockedStatusChanged(network: Network, blocked: Boolean) {
                    super.onBlockedStatusChanged(network, blocked)
                    trySendBlocking(NetworkEvent.OnBlockedStatusChanged(network, blocked))
                }

                override fun onLosing(network: Network, maxMsToLive: Int) {
                    super.onLosing(network, maxMsToLive)
                    trySendBlocking(NetworkEvent.OnLosing(network, maxMsToLive))
                }

                override fun onLost(network: Network) {
                    super.onLost(network)
                    trySendBlocking(NetworkEvent.OnLost(network))
                }

                override fun onUnavailable() {
                    super.onUnavailable()
                    trySendBlocking(NetworkEvent.OnUnavailable)
                }
            }
        registerDefaultNetworkCallback(callback)

        awaitClose { unregisterNetworkCallback(callback) }
    }
