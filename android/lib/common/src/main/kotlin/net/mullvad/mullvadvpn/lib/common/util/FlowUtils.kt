@file:Suppress("MagicNumber")

package net.mullvad.mullvadvpn.lib.common.util

import kotlinx.coroutines.Deferred
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.emitAll
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.launch

inline fun <T> Flow<T>.onFirst(crossinline action: suspend (T) -> Unit): Flow<T> {
    return flow {
        var first = true
        collect { value ->
            if (first) {
                action(value)
                first = false
            }
            emit(value)
        }
    }
}

@Suppress("LongParameterList")
inline fun <T1, T2, T3, T4, T5, T6, R> combine(
    flow: Flow<T1>,
    flow2: Flow<T2>,
    flow3: Flow<T3>,
    flow4: Flow<T4>,
    flow5: Flow<T5>,
    flow6: Flow<T6>,
    crossinline transform: suspend (T1, T2, T3, T4, T5, T6) -> R,
): Flow<R> {
    return combine(flow, flow2, flow3, flow4, flow5, flow6) { args: Array<*> ->
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

@Suppress("LongParameterList")
inline fun <T1, T2, T3, T4, T5, T6, T7, R> combine(
    flow: Flow<T1>,
    flow2: Flow<T2>,
    flow3: Flow<T3>,
    flow4: Flow<T4>,
    flow5: Flow<T5>,
    flow6: Flow<T6>,
    flow7: Flow<T7>,
    crossinline transform: suspend (T1, T2, T3, T4, T5, T6, T7) -> R,
): Flow<R> {
    return combine(flow, flow2, flow3, flow4, flow5, flow6, flow7) { args: Array<*> ->
        @Suppress("UNCHECKED_CAST")
        transform(
            args[0] as T1,
            args[1] as T2,
            args[2] as T3,
            args[3] as T4,
            args[4] as T5,
            args[5] as T6,
            args[6] as T7,
        )
    }
}

@Suppress("LongParameterList")
inline fun <T1, T2, T3, T4, T5, T6, T7, T8, R> combine(
    flow: Flow<T1>,
    flow2: Flow<T2>,
    flow3: Flow<T3>,
    flow4: Flow<T4>,
    flow5: Flow<T5>,
    flow6: Flow<T6>,
    flow7: Flow<T7>,
    flow8: Flow<T8>,
    crossinline transform: suspend (T1, T2, T3, T4, T5, T6, T7, T8) -> R,
): Flow<R> {
    return combine(flow, flow2, flow3, flow4, flow5, flow6, flow7, flow8) { args: Array<*> ->
        @Suppress("UNCHECKED_CAST")
        transform(
            args[0] as T1,
            args[1] as T2,
            args[2] as T3,
            args[3] as T4,
            args[4] as T5,
            args[5] as T6,
            args[6] as T7,
            args[7] as T8,
        )
    }
}

@Suppress("LongParameterList")
inline fun <T1, T2, T3, T4, T5, T6, T7, T8, T9, R> combine(
    flow: Flow<T1>,
    flow2: Flow<T2>,
    flow3: Flow<T3>,
    flow4: Flow<T4>,
    flow5: Flow<T5>,
    flow6: Flow<T6>,
    flow7: Flow<T7>,
    flow8: Flow<T8>,
    flow9: Flow<T9>,
    crossinline transform: suspend (T1, T2, T3, T4, T5, T6, T7, T8, T9) -> R,
): Flow<R> {
    return combine(flow, flow2, flow3, flow4, flow5, flow6, flow7, flow8, flow9) { args: Array<*> ->
        @Suppress("UNCHECKED_CAST")
        transform(
            args[0] as T1,
            args[1] as T2,
            args[2] as T3,
            args[3] as T4,
            args[4] as T5,
            args[5] as T6,
            args[6] as T7,
            args[7] as T8,
            args[8] as T9,
        )
    }
}

@OptIn(ExperimentalCoroutinesApi::class)
fun <T> Deferred<T>.getOrDefault(default: T) =
    try {
        getCompleted()
    } catch (e: IllegalStateException) {
        default
    }

fun <T> Flow<T>.withPrev(): Flow<Pair<T, T?>> = flow {
    var prev: T? = null
    collect { curr ->
        emit(curr to prev)
        prev = curr
    }
}

/**
 * Launches [action] in a child coroutine when collection starts.
 *
 * Like [onStart], but [action] is launched instead of awaited, so it runs concurrently with the
 * collection. It is canceled when the collection is canceled.
 *
 * Typical usage with stateIn in a ViewModel:
 * ```
 * val uiState: StateFlow<UiStateProfile> = repository.observeProfile()
 *     .launchOnStart { repository.refresh() }
 *     .stateIn(
 *         scope = viewModelScope,
 *         started = SharingStarted.WhileSubscribed(5_000),
 *         initialValue = UiStateProfile.INITIAL,
 *     )
 * ```
 *
 * The refresh function is launched in viewModelScope with the first subscriber of uiState and is
 * canceled when the stateIn stops, in this case 5 seconds after the last subscriber leaves because
 * of SharingStarted.WhileSubscribed(5_000).
 */
fun <T> Flow<T>.launchOnStart(action: suspend () -> Unit): Flow<T> = flow {
    coroutineScope {
        launch { action() }
        emitAll(this@launchOnStart)
    }
}
