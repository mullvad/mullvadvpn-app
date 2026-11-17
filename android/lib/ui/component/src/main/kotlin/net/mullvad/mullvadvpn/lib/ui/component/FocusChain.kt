package net.mullvad.mullvadvpn.lib.ui.component

import androidx.compose.foundation.relocation.BringIntoViewRequester
import androidx.compose.foundation.relocation.bringIntoViewRequester
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged

/**
 * Allows creating an explicit linear focus ordering for a group of components. This is necessary
 * because `AnimatedVisibilityItem` measures its content with an animated height, which breaks the
 * default focus navigation when navigating in from above.
 *
 * A `null` entry in the order is a gap where the default spatial search is used.
 */
class FocusChain {
    private val requesters = mutableMapOf<Any, FocusRequester>()
    private var order: List<Any?> = emptyList()

    fun update(order: List<Any?>) {
        this.order = order
    }

    fun requester(key: Any): FocusRequester = requesters.getOrPut(key) { FocusRequester() }

    fun previous(key: Any): FocusRequester? =
        order.indexOf(key).takeIf { it > 0 }?.let { order[it - 1]?.let(::requester) }

    fun next(key: Any): FocusRequester? =
        order
            .indexOf(key)
            .takeIf { it >= 0 && (it < order.lastIndex) }
            ?.let { order[it + 1]?.let(::requester) }
}

private val LocalFocusChain = staticCompositionLocalOf<FocusChain?> { null }

@Composable
fun ProvideFocusChain(order: List<Any?>, content: @Composable () -> Unit) {
    val chain = remember { FocusChain() }
    chain.update(order)
    CompositionLocalProvider(LocalFocusChain provides chain, content = content)
}

/** Declares the D-pad position of one focusable. Must be used inside [ProvideFocusChain]. */
@Composable
fun Modifier.focusChainItem(key: Any): Modifier {
    val chain =
        LocalFocusChain.current ?: error("focusChainItem must be used inside ProvideFocusChain")
    return focusRequester(chain.requester(key)).focusProperties {
        chain.previous(key)?.let { up = it }
        chain.next(key)?.let { down = it }
    }
}

/**
 * Keeps a focused item visible when rows are inserted or removed above it.
 *
 * `trigger` is the value that changes when the list changes (usually its size). The initial value
 * does not cause a scroll.
 */
@Composable
fun Modifier.keepFocusedInView(trigger: Any): Modifier {
    val bringIntoViewRequester = remember { BringIntoViewRequester() }
    var isFocused by remember { mutableStateOf(false) }
    var previousTrigger by remember { mutableStateOf(trigger) }

    LaunchedEffect(trigger) {
        val listChanged = trigger != previousTrigger
        previousTrigger = trigger
        if (listChanged && isFocused) bringIntoViewRequester.bringIntoView()
    }

    return this.bringIntoViewRequester(bringIntoViewRequester).onFocusChanged {
        isFocused = it.isFocused
    }
}
