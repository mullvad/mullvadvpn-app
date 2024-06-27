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
fun <T1, T2> dropUnlessResumed(block: (T1, T2) -> Unit): (T1, T2) -> Unit {
    val lifecycle = LocalLifecycleOwner.current.lifecycle
    return dropUnlessResumed(lifecycle.currentState, block)
}

fun <T1, T2> dropUnlessResumed(state: Lifecycle.State, block: (T1, T2) -> Unit): (T1, T2) -> Unit =
    state.runOnAtLeast(Lifecycle.State.RESUMED, block)

fun <T1, T2> Lifecycle.State.runOnAtLeast(
    expectedState: Lifecycle.State,
    block: (T1, T2) -> Unit
): (T1, T2) -> Unit {
    return { t1, t2 ->
        if (isAtLeast(expectedState)) {
            block(t1, t2)
        }
    }
}

@Composable
fun <T1, T2, T3> dropUnlessResumed(block: (T1, T2, T3) -> Unit): (T1, T2, T3) -> Unit {
    val lifecycle = LocalLifecycleOwner.current.lifecycle
    return dropUnlessResumed(lifecycle.currentState, block)
}

fun <T1, T2, T3> dropUnlessResumed(
    state: Lifecycle.State,
    block: (T1, T2, T3) -> Unit
): (T1, T2, T3) -> Unit = state.runOnAtLeast(Lifecycle.State.RESUMED, block)

fun <T1, T2, T3> Lifecycle.State.runOnAtLeast(
    expectedState: Lifecycle.State,
    block: (T1, T2, T3) -> Unit
): (T1, T2, T3) -> Unit {
    return { t1, t2, t3 ->
        if (isAtLeast(expectedState)) {
            block(t1, t2, t3)
        }
    }
}
