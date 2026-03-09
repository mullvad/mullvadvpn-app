package net.mullvad.mullvadvpn.core.animation

import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.navigation3.runtime.metadata
import androidx.navigation3.ui.NavDisplay

fun loginTransition(shouldFadeOut: () -> Boolean): Map<String, Any> = metadata {

    // Fade in the pushed screen together with optionally fading out the current screen
    put(NavDisplay.TransitionKey) {
        fadeIn(tween(TRANSITION_DEFAULT_DURATION_MS)) togetherWith
            if (shouldFadeOut()) fadeOut(tween(TRANSITION_DEFAULT_DURATION_MS))
            else ExitTransition.None
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
