package net.mullvad.mullvadvpn.core.animation

import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.togetherWith
import androidx.navigation3.runtime.metadata
import androidx.navigation3.ui.NavDisplay

fun accountTransition(): Map<String, Any> = metadata {

    // Fade and scale in pushed screen
    put(NavDisplay.TransitionKey) {
        fadeIn(tween(TRANSITION_DEFAULT_DURATION_MS)) +
            scaleIn(
                animationSpec = tween(TRANSITION_DEFAULT_DURATION_MS),
                initialScale = ENTER_TRANSITION_SCALE_IN_FACTOR,
            ) togetherWith ExitTransition.None
    }

    // Fade out the popped screen and fade in the new top screen
    put(NavDisplay.PopTransitionKey) {
        fadeIn(tween(TRANSITION_DEFAULT_DURATION_MS)) togetherWith
            fadeOut(tween(TRANSITION_DEFAULT_DURATION_MS))
    }

    put(NavDisplay.PredictivePopTransitionKey) {
        fadeIn(tween(TRANSITION_DEFAULT_DURATION_MS)) togetherWith
            fadeOut(tween(TRANSITION_DEFAULT_DURATION_MS))
    }
}
