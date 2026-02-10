package net.mullvad.mullvadvpn.common.compose

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.lifecycle.flowWithLifecycle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow

// This function will restart collection on Start/Stop events, e.g if the user navigates to home
// screen collection will stop, and then be restarted when the user opens the app again
@Composable
inline fun <T> CollectSideEffectWithLifecycle(
    sideEffect: Flow<T>,
    minActiveState: Lifecycle.State = Lifecycle.State.STARTED,
    key: Any? = Unit,
    crossinline collector: suspend CoroutineScope.(T) -> Unit,
) {
    val lifecycleOwner = LocalLifecycleOwner.current

    LaunchedEffect(lifecycleOwner, key) {
        sideEffect.flowWithLifecycle(lifecycleOwner.lifecycle, minActiveState).collect {
            collector(it)
        }
    }
}
