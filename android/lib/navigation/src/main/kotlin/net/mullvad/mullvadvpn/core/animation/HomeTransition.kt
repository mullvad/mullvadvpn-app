package net.mullvad.mullvadvpn.core.animation

import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.togetherWith
import androidx.navigation3.runtime.metadata
import androidx.navigation3.ui.NavDisplay

fun homeTransition(shouldFadeIn: () -> Boolean): Map<String, Any> = metadata {

    // Optionally fade in the pushed screen
    put(NavDisplay.TransitionKey) {
        val enter =
            if (shouldFadeIn()) fadeIn(tween(TRANSITION_DEFAULT_DURATION_MS))
            else EnterTransition.None

        enter togetherWith ExitTransition.None
    }

    put(NavDisplay.PopTransitionKey) { EnterTransition.None togetherWith ExitTransition.None }

    put(NavDisplay.PredictivePopTransitionKey) {
        EnterTransition.None togetherWith ExitTransition.None
    }
}
