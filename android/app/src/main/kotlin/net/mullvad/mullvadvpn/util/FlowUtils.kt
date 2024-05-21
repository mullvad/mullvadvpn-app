@file:Suppress("MagicNumber")

package net.mullvad.mullvadvpn.util

import kotlinx.coroutines.Deferred
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.retryWhen
import kotlinx.coroutines.withTimeoutOrNull

inline fun <T1, T2, T3, T4, T5, T6, R> combine(
    flow: Flow<T1>,
    flow2: Flow<T2>,
    flow3: Flow<T3>,
    flow4: Flow<T4>,
    flow5: Flow<T5>,
    flow6: Flow<T6>,
    crossinline transform: suspend (T1, T2, T3, T4, T5, T6) -> R
): Flow<R> {
    return kotlinx.coroutines.flow.combine(flow, flow2, flow3, flow4, flow5, flow6) { args: Array<*>
        ->
        @Suppress("UNCHECKED_CAST")
        transform(
            args[0] as T1,
            args[1] as T2,
            args[2] as T3,
            args[3] as T4,
            args[4] as T5,
            args[5] as T6,
        )
    }
}

inline fun <T1, T2, T3, T4, T5, T6, T7, R> combine(
    flow: Flow<T1>,
    flow2: Flow<T2>,
    flow3: Flow<T3>,
    flow4: Flow<T4>,
    flow5: Flow<T5>,
    flow6: Flow<T6>,
    flow7: Flow<T7>,
    crossinline transform: suspend (T1, T2, T3, T4, T5, T6, T7) -> R
): Flow<R> {
    return kotlinx.coroutines.flow.combine(flow, flow2, flow3, flow4, flow5, flow6, flow7) {
        args: Array<*> ->
        @Suppress("UNCHECKED_CAST")
        transform(
            args[0] as T1,
            args[1] as T2,
            args[2] as T3,
            args[3] as T4,
            args[4] as T5,
            args[5] as T6,
            args[6] as T7
        )
    }
}

inline fun <T1, T2, T3, T4, T5, T6, T7, T8, R> combine(
    flow: Flow<T1>,
    flow2: Flow<T2>,
    flow3: Flow<T3>,
    flow4: Flow<T4>,
    flow5: Flow<T5>,
    flow6: Flow<T6>,
    flow7: Flow<T7>,
    flow8: Flow<T8>,
    crossinline transform: suspend (T1, T2, T3, T4, T5, T6, T7, T8) -> R
): Flow<R> {
    return kotlinx.coroutines.flow.combine(flow, flow2, flow3, flow4, flow5, flow6, flow7, flow8) {
        args: Array<*> ->
        @Suppress("UNCHECKED_CAST")
        transform(
            args[0] as T1,
            args[1] as T2,
            args[2] as T3,
            args[3] as T4,
            args[4] as T5,
            args[5] as T6,
            args[6] as T7,
            args[7] as T8
        )
    }
}

suspend inline fun <T> Deferred<T>.awaitWithTimeoutOrNull(timeout: Long) =
    withTimeoutOrNull(timeout) { await() }

fun <T> Deferred<T>.getOrDefault(default: T) =
    try {
        getCompleted()
    } catch (e: IllegalStateException) {
        default
    }

@Suppress("UNCHECKED_CAST")
suspend inline fun <T> Flow<T>.retryWithExponentialBackOff(
    maxAttempts: Int,
    initialBackOffDelay: Long,
    backOffDelayFactor: Long,
    crossinline predicate: (T) -> Boolean,
): Flow<T> =
    map {
            if (predicate(it)) {
                throw ExceptionWrapper(it as Any)
            }
            it
        }
        .retryWhen { _, attempt ->
            if (attempt >= maxAttempts) {
                return@retryWhen false
            }
            val backOffDelay = initialBackOffDelay * backOffDelayFactor.pow(attempt.toInt())
            delay(backOffDelay)
            true
        }
        .catch {
            if (it is ExceptionWrapper) {
                this.emit(it.item as T)
            } else {
                throw it
            }
        }

class ExceptionWrapper(val item: Any) : Throwable()

