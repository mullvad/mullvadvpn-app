package net.mullvad.mullvadvpn.ipc

import kotlin.reflect.KClass

interface MessageHandler<T : Any> {
    fun <V : T> registerHandler(variant: KClass<V>, handler: (V) -> Unit)
}
