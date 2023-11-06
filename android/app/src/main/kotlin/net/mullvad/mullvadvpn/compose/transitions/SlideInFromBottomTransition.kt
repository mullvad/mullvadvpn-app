package net.mullvad.mullvadvpn.compose.transitions

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.navigation.NavBackStackEntry
import com.ramcosta.composedestinations.spec.DestinationStyle

object SlideInFromBottomTransition : DestinationStyle.Animated {
    override fun AnimatedContentTransitionScope<NavBackStackEntry>.enterTransition():
        EnterTransition {
        return slideInVertically(initialOffsetY = { it })
    }

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popEnterTransition() = null

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popExitTransition():
        ExitTransition {
        return slideOutVertically(targetOffsetY = { it })
    }
}
