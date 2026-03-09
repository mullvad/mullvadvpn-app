package net.mullvad.mullvadvpn.core.nav3

import androidx.compose.runtime.Composable
import androidx.compose.runtime.ProvidableCompositionLocal
import androidx.compose.runtime.ProvidedValue
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.saveable.Saver
import androidx.compose.runtime.saveable.rememberSaveable
import kotlinx.serialization.Serializable

/** Local for storing results in a [ResultStore] */
object LocalResultStore {
    private val LocalResultStore: ProvidableCompositionLocal<ResultStore?> = compositionLocalOf {
        null
    }

    /** The current [ResultStore] */
    val current: ResultStore
        @Composable get() = LocalResultStore.current ?: error("No ResultStore has been provided")

    /** Provides a [ResultStore] to the composition */
    infix fun provides(store: ResultStore): ProvidedValue<ResultStore?> {
        return LocalResultStore.provides(store)
    }
}

@Composable
fun rememberResultStore(): ResultStore {
    return rememberSaveable(saver = resultStoreSaver()) { ResultStore() }
}

/** A store for passing results between multiple sets of screens. */
class ResultStore {

    /** Map from the result key to a mutable state of the result. */
    val resultStateMap = mutableStateMapOf<String, ResultStoreStateHolder>()

    /** Retrieves and consumes the current result of the given resultKey. */
    inline fun <reified T : NavigationResult> consumeResult(): T? {
        val key = T::class.toString()
        val result = resultStateMap[key]?.value as? T
        if (result != null) resultStateMap[key]?.value = null
        return result
    }

    /** Sets the result for the given resultKey. */
    inline fun <reified T : NavigationResult> setResult(result: T) {
        val key = T::class.toString()
        resultStateMap[key] = ResultStoreStateHolder(result)
    }
}

// The purpose of this class is to avoid unnecessary recompositions after a result has been
// consumed. This is done by not removing the key from `resultStateMap` after the
// value is consumed (which would trigger a recomposition), but instead setting
// the `ResultStoreStateHolder` value to null.
class ResultStoreStateHolder(var value: NavigationResult?)

/** Saver to save and restore the ResultStore across config change and process death. */
private fun resultStoreSaver(): Saver<ResultStore, *> =
    Saver(
        save = { resultStore -> resultStore.resultStateMap.mapValues { it.value.value as Any? } },
        restore = {
            ResultStore().apply {
                resultStateMap.putAll(
                    it.mapValues { entry ->
                        ResultStoreStateHolder(entry.value as? NavigationResult)
                    }
                )
            }
        },
    )
