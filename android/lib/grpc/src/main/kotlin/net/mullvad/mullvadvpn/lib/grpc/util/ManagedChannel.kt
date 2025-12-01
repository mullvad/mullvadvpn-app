package net.mullvad.mullvadvpn.lib.grpc.util

import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import net.mullvad.mullvadvpn.lib.grpc.GrpcConnectivityState
import okhttp3.OkHttpClient

internal fun OkHttpClient.connectivityFlow(): Flow<GrpcConnectivityState> {
    return callbackFlow {
        send(GrpcConnectivityState.Ready)
        /*var currentState = getState(false)

        while (isActive) {
            // Check that we are active before sending
            send(currentState)
            currentState = suspendCancellableCoroutine {
                notifyWhenStateChanged(currentState) {
                    // If we are cancelled we will just log
                    it.resume(getState(false)) { cause, value, _ ->
                        Logger.w("Resume while cancelled, value: $value", cause)
                    }
                }
            }
        }*/
        awaitClose {}
    }
}
