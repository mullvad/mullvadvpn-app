package net.mullvad.mullvadvpn.compose.extensions

import androidx.compose.runtime.Composable
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.compose.LocalLifecycleOwner
import co.touchlab.kermit.Logger

fun Lifecycle.State.dropUnlessResumed(block: () -> Unit) =
    runOnAtLeast(Lifecycle.State.RESUMED, block)

fun Lifecycle.State.runOnAtLeast(expectedState: Lifecycle.State, block: () -> Unit) {
    if (isAtLeast(expectedState)) {
        block()
    }
}

@Composable
fun <T> dropUnlessResumed(block: (T) -> Unit): (T) -> Unit {
    val lifecycleOwner = LocalLifecycleOwner.current
    return dropUnlessResumed(lifecycleOwner, block)
}

fun <T> dropUnlessResumed(lifecycleOwner: LifecycleOwner, block: (T) -> Unit): (T) -> Unit =
    lifecycleOwner.runOnAtLeast(Lifecycle.State.RESUMED, block)

fun <T> LifecycleOwner.runOnAtLeast(
    expectedState: Lifecycle.State,
    block: (T) -> Unit,
): (T) -> Unit {
    return {
        if (lifecycle.currentState.isAtLeast(expectedState)) {
            block(it)
        } else {
            Logger.v("runOnAtLeast skipped due to ${lifecycle.currentState}<${expectedState}")
        }
    }
}

@Composable
fun <T, T2> dropUnlessResumed(block: (T, T2) -> Unit): (T, T2) -> Unit {
    val lifecycleOwner = LocalLifecycleOwner.current
    return dropUnlessResumed(lifecycleOwner, block)
}

fun <T, T2> dropUnlessResumed(
    lifecycleOwner: LifecycleOwner,
    block: (T, T2) -> Unit,
): (T, T2) -> Unit = lifecycleOwner.runOnAtLeast(Lifecycle.State.RESUMED, block)

fun <T, T2> LifecycleOwner.runOnAtLeast(
    expectedState: Lifecycle.State,
    block: (T, T2) -> Unit,
): (T, T2) -> Unit {
    return { t, t1 ->
        if (lifecycle.currentState.isAtLeast(expectedState)) {
            block(t, t1)
        } else {
            Logger.v("runOnAtLeast skipped due to ${lifecycle.currentState}<${expectedState}")
        }
    }
}
