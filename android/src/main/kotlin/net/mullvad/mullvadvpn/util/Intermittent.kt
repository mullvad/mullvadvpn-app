package net.mullvad.mullvadvpn.util

import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.sync.withPermit
import net.mullvad.talpid.util.EventNotifier

// Wrapper to allow awaiting for intermittent values.
//
// Wraps a property that is changed from time to time and that can become unavailable (null). This
// behaves in a way similar to `CompletableDeferred`, but the value can be set and reset multiple
// times.
//
// Calling `await` will either provide the value if it's available, or suspend until it becomes
// available and then return it.
//
// Calling `update` will set the internal value after it guarantees that no other coroutine is
// currently reading the value (through a permit from the semaphore). After the value is set, it
// provides a permit to the semaphore so that suspended coroutines can use the new value.
//
// Extra initialization can be done on the intermittent value when it becomes available and before
// it is provided to the awaiting coroutines, through the use of listener callbacks. These are
// called after the value is updated but before it is made available to the coroutines.
class Intermittent<T> {
    private val notifier = EventNotifier<T?>(null)
    private val semaphore = Semaphore(1, 1)
    private val writeLock = Mutex()

    private var updateJob: Job? = null
    private var value by notifier.notifiable()

    // When the internal value is updated, listeners can be notified before the awaiting coroutines
    // resume execution. This allows performing any extra initialization before the value is made
    // available for usage.
    fun registerListener(id: Any, listener: (T?) -> Unit) = notifier.subscribe(id, listener)
    fun unregisterListener(id: Any) = notifier.unsubscribe(id)

    suspend fun await(): T {
        return semaphore.withPermit { value!! }
    }

    suspend fun update(newValue: T?) {
        writeLock.withLock {
            if (newValue != value) {
                if (value != null) {
                    semaphore.acquire()
                }

                // This will trigger the listeners to run before the awaiting coroutines resume
                value = newValue

                if (newValue != null) {
                    semaphore.release()
                }
            }
        }
    }

    // Helper method that provides a simple way to change the wrapped value.
    // 
    // The method returns a property delegate that will spawn a coroutine to update the wrapped
    // value every time the property is written to.
    fun source() = observable<T?>(null) { _, _, newValue ->
        synchronized(this@Intermittent) {
            val previousUpdate = updateJob

            updateJob = GlobalScope.launch(Dispatchers.Default) {
                previousUpdate?.join()
                update(newValue)
            }
        }
    }

    fun onDestroy() {
        notifier.unsubscribeAll()
    }
}
