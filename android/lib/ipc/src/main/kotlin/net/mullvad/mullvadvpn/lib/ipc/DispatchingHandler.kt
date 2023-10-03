package net.mullvad.mullvadvpn.lib.ipc

import android.os.Handler
import android.os.Looper
import android.os.Message
import android.util.Log
import java.util.concurrent.locks.ReentrantReadWriteLock
import kotlin.concurrent.withLock
import kotlin.reflect.KClass
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow

class DispatchingHandler<T : Any>(looper: Looper, private val extractor: (Message) -> T?) :
    Handler(looper), MessageDispatcher<T> {
    private val handlers = HashMap<KClass<out T>, (T) -> Unit>()
    private val lock = ReentrantReadWriteLock()

    private val _parsedMessages = MutableSharedFlow<T>(extraBufferCapacity = 1)
    val parsedMessages = _parsedMessages.asSharedFlow()

    @Deprecated("Use parsedMessages instead.")
    override fun <V : T> registerHandler(variant: KClass<V>, handler: (V) -> Unit) {
        lock.writeLock().withLock {
            handlers.put(variant) { instance -> @Suppress("UNCHECKED_CAST") handler(instance as V) }
        }
    }

    override fun handleMessage(message: Message) {
        lock.readLock().withLock {
            val instance = extractor(message)

            if (instance != null) {
                val handler = handlers.get(instance::class)

                handler?.invoke(instance)
                _parsedMessages.tryEmit(instance)
            } else {
                Log.e("mullvad", "Dispatching handler received an unexpected message")
            }
        }
    }

    fun onDestroy() {
        lock.writeLock().withLock { handlers.clear() }

        removeCallbacksAndMessages(null)
    }
}
