package net.mullvad.mullvadvpn.lib.ipc

import kotlin.reflect.KClass

interface MessageDispatcher<T : Any> {
    fun <V : T> registerHandler(variant: KClass<V>, handler: (V) -> Unit)
}
