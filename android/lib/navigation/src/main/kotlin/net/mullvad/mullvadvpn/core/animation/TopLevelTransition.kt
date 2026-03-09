package net.mullvad.mullvadvpn.core.animation

import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.togetherWith
import androidx.navigation3.runtime.metadata
import androidx.navigation3.ui.NavDisplay

fun topLevelTransition(): Map<String, Any> = metadata {

    // Fade and scale in the pushed screen
    put(NavDisplay.TransitionKey) {
        fadeIn(tween(TRANSITION_DEFAULT_DURATION_MS)) +
            scaleIn(initialScale = ENTER_TRANSITION_SCALE_IN_FACTOR) togetherWith
            ExitTransition.None
    }

    // Fade out the popped screen
    put(NavDisplay.PopTransitionKey) {
        EnterTransition.None togetherWith fadeOut(tween(TRANSITION_DEFAULT_DURATION_MS))
    }

    // Slide the popped screen out from start to end
    put(NavDisplay.PredictivePopTransitionKey) {
        EnterTransition.None togetherWith fadeOut(tween(TRANSITION_DEFAULT_DURATION_MS))
    }
}
