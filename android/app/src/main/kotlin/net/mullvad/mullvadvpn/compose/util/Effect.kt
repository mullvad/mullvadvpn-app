package net.mullvad.mullvadvpn.compose.util

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.flowWithLifecycle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow

@Composable
inline fun <T> LaunchedEffectCollect(
    sideEffect: Flow<T>,
    key: Any = Unit,
    crossinline collector: suspend CoroutineScope.(T) -> Unit
) {
    LaunchedEffect(key) { sideEffect.collect { collector(it) } }
}

// This function will restart collection on Start/Stop events, e.g if the user navigates to home
// screen collection will stop, and then be restarted when the user opens the app again
@Composable
inline fun <T> CollectSideEffectWithLifecycle(
    sideEffect: Flow<T>,
    minActiveState: Lifecycle.State = Lifecycle.State.STARTED,
    key: Any? = Unit,
    crossinline collector: suspend CoroutineScope.(T) -> Unit
) {
    val lifecycleOwner = LocalLifecycleOwner.current

    LaunchedEffect(lifecycleOwner, key) {
        sideEffect.flowWithLifecycle(lifecycleOwner.lifecycle, minActiveState).collect {
            collector(it)
        }
    }
}
