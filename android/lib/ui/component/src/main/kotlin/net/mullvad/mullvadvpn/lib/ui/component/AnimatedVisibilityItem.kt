package net.mullvad.mullvadvpn.lib.ui.component

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.MutableTransitionState
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember

/**
 * This composable can be used enable entry/exit animations for Column (not LazyColumn) items.
 * `showEntryAnimation` - whether `content` should animate in when added or not.
 * `onRemoveConfirmed` - called when the exit animation is done and it is safe to remove the item.
 * `content` - the column item content lambda. The content lambda should invoke the
 * `onRequestRemove` parameter when it wants to remove the content.
 */
@Composable
fun AnimatedVisibilityItem(
    showEntryAnimation: Boolean = true,
    onRemoveConfirmed: () -> Unit,
    content: @Composable (onRequestRemove: () -> Unit) -> Unit,
) {
    // Default initial state to true so it animates in when first added.
    val visibleState = remember {
        MutableTransitionState(!showEntryAnimation).apply { targetState = true }
    }

    // Check if the exit animation has finished
    if (visibleState.isIdle && !visibleState.currentState) {
        LaunchedEffect(visibleState) { onRemoveConfirmed() }
    }

    AnimatedVisibility(
        visibleState = visibleState,
        enter = fadeIn() + expandVertically(),
        exit = fadeOut() + shrinkVertically(),
    ) {
        content { visibleState.targetState = false }
    }
}
