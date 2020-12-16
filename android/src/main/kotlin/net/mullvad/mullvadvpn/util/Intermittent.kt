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
// provides a premit to the semaphore so that suspended coroutines can use the new value.
class Intermittent<T> {
    private val semaphore = Semaphore(1, 1)
    private val writeLock = Mutex()

    private var updateJob: Job? = null
    private var value: T? = null

    suspend fun await(): T {
        return semaphore.withPermit { value!! }
    }

    suspend fun update(newValue: T?) {
        writeLock.withLock {
            if (newValue != value) {
                if (value != null) {
                    semaphore.acquire()
                }

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
}
