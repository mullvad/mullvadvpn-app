package net.mullvad.mullvadvpn.ui.serviceconnection

import kotlin.reflect.KClass
import kotlinx.coroutines.flow.Flow
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.Request

interface MessageHandler {
    fun <R : Event> events(klass: KClass<R>): Flow<R>

    fun trySendRequest(request: Request): Boolean
}

inline fun <reified R : Event> MessageHandler.events(): Flow<R> {
    return this.events(R::class)
}
