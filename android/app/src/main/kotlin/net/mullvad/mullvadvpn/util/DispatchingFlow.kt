package net.mullvad.mullvadvpn.util

import java.util.concurrent.ConcurrentHashMap
import kotlin.reflect.KClass
import kotlinx.coroutines.InternalCoroutinesApi
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedSendChannelException
import kotlinx.coroutines.channels.SendChannel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.FlowCollector
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.consumeAsFlow

class DispatchingFlow<T : Any>(private val upstream: Flow<T>) : Flow<T> {
    private val subscribers = ConcurrentHashMap<KClass<out T>, SendChannel<T>>()

    fun <V : T> subscribe(variant: KClass<V>, capacity: Int = Channel.CONFLATED): Flow<V> {
        val channel = Channel<V>(capacity)

        // This is safe because `collect` will only send to this channel if the instance class is V
        @Suppress("UNCHECKED_CAST")
        subscribers[variant] = channel as SendChannel<T>

        return channel.consumeAsFlow()
    }

    fun <V : T> unsubscribe(variant: KClass<V>) = subscribers.remove(variant)

    @InternalCoroutinesApi
    override suspend fun collect(collector: FlowCollector<T>) {
        upstream.collect { event ->
            try {
                subscribers[event::class]?.send(event)
            } catch (closedException: ClosedSendChannelException) {
                subscribers.remove(event::class)
            }

            collector.emit(event)
        }

        subscribers.clear()
    }
}

fun <T : Any> Flow<T>.dispatchTo(configureSubscribers: DispatchingFlow<T>.() -> Unit) =
    DispatchingFlow(this).also(configureSubscribers)
