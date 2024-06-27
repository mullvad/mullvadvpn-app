package net.mullvad.mullvadvpn.compose.extensions

import androidx.compose.runtime.Composable
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LocalLifecycleOwner

fun Lifecycle.State.dropUnlessResumed(block: () -> Unit) =
    runOnAtLeast(Lifecycle.State.RESUMED, block)

fun Lifecycle.State.runOnAtLeast(expectedState: Lifecycle.State, block: () -> Unit) {
    if (isAtLeast(expectedState)) {
        block()
    }
}

@Composable
fun <T> dropUnlessResumed(block: (T) -> Unit): (T) -> Unit {
    val lifecycle = LocalLifecycleOwner.current.lifecycle
    return dropUnlessResumed(lifecycle.currentState, block)
}

fun <T> dropUnlessResumed(state: Lifecycle.State, block: (T) -> Unit): (T) -> Unit =
    state.runOnAtLeast(Lifecycle.State.RESUMED, block)

fun <T> Lifecycle.State.runOnAtLeast(
    expectedState: Lifecycle.State,
    block: (T) -> Unit
): (T) -> Unit {
    return {
        if (isAtLeast(expectedState)) {
            block(it)
        }
    }
}

@Composable
fun <T, T2> dropUnlessResumed(block: (T, T2) -> Unit): (T, T2) -> Unit {
    val lifecycle = LocalLifecycleOwner.current.lifecycle
    return dropUnlessResumed(lifecycle.currentState, block)
}

fun <T, T2> dropUnlessResumed(state: Lifecycle.State, block: (T, T2) -> Unit): (T, T2) -> Unit =
    state.runOnAtLeast(Lifecycle.State.RESUMED, block)

fun <T, T2> Lifecycle.State.runOnAtLeast(
    expectedState: Lifecycle.State,
    block: (T, T2) -> Unit
): (T, T2) -> Unit {
    return { t, t1 ->
        if (isAtLeast(expectedState)) {
            block(t, t1)
        }
    }
}
