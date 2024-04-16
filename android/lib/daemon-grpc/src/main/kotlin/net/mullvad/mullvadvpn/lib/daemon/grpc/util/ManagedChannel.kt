package net.mullvad.mullvadvpn.lib.daemon.grpc.util

import io.grpc.ConnectivityState
import io.grpc.ManagedChannel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.isActive
import kotlin.coroutines.resume
import kotlin.coroutines.suspendCoroutine

internal fun ManagedChannel.connectivityFlow(): Flow<ConnectivityState> {
    return callbackFlow {
        var currentState = getState(false)
        send(currentState)

        while (isActive) {
            currentState =
                suspendCoroutine<ConnectivityState> {
                    notifyWhenStateChanged(currentState) { it.resume(getState(false)) }
                }
            send(currentState)
        }
    }
}

