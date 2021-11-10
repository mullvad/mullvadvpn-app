package net.mullvad.mullvadvpn.service.endpoint

import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.util.safeOffer

class ForegroundRequestHandler(val endpoint: ServiceEndpoint) {
    fun foregroundRequests(): Flow<Boolean> = callbackFlow {
        endpoint.dispatcher.registerHandler(Request.SetForcedForeground::class) { request ->
            safeOffer(request.doForceForeground)
        }
        awaitClose {
            // TODO: Not able to unregister...
        }
    }
}
