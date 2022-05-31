package net.mullvad.talpid.util

import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.callbackFlow

fun <T> EventNotifier<T>.callbackFlowFromSubscription(id: Any) = callbackFlow {
    this@callbackFlowFromSubscription.subscribe(id) {
        this.trySend(it)
    }
    awaitClose {
        this@callbackFlowFromSubscription.unsubscribe(id)
    }
}
