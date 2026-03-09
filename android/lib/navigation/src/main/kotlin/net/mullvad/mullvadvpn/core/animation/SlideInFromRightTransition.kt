package net.mullvad.mullvadvpn.core.animation

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.navigation3.runtime.metadata
import androidx.navigation3.ui.NavDisplay

fun slideInHorizontalTransition(): Map<String, Any> = metadata {

    // Slide the pushed screen in from end to start
    put(NavDisplay.TransitionKey) {
        fadeIn(tween(TRANSITION_DEFAULT_DURATION_MS)) +
            slideIntoContainer(
                animationSpec = tween(TRANSITION_DEFAULT_DURATION_MS),
                towards = AnimatedContentTransitionScope.SlideDirection.Start,
                initialOffset = { (it * ENTER_TRANSITION_SLIDE_FACTOR).toInt() },
            ) togetherWith ExitTransition.None
    }

    // Slide the popped screen out from start to end
    put(NavDisplay.PopTransitionKey) {
        EnterTransition.None togetherWith
            slideOutOfContainer(
                animationSpec = tween(TRANSITION_DEFAULT_DURATION_MS),
                towards = AnimatedContentTransitionScope.SlideDirection.End,
                targetOffset = { (it * ENTER_TRANSITION_SLIDE_FACTOR).toInt() },
            ) + fadeOut(tween(TRANSITION_DEFAULT_DURATION_MS))
    }

    put(NavDisplay.PredictivePopTransitionKey) {
        EnterTransition.None togetherWith
            slideOutOfContainer(
                animationSpec = tween(TRANSITION_DEFAULT_DURATION_MS),
                towards = AnimatedContentTransitionScope.SlideDirection.End,
                targetOffset = { (it * ENTER_TRANSITION_SLIDE_FACTOR).toInt() },
            ) + fadeOut(tween(TRANSITION_DEFAULT_DURATION_MS))
    }
}
