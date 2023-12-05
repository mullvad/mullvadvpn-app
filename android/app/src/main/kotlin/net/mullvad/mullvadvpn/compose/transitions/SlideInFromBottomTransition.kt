package net.mullvad.mullvadvpn.compose.transitions

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.snap
import androidx.compose.animation.fadeOut
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

object SelectLocationTransition : DestinationStyle.Animated {
    override fun AnimatedContentTransitionScope<NavBackStackEntry>.enterTransition():
        EnterTransition {
        return slideInVertically(initialOffsetY = { it })
    }

    // TODO temporary hack until we have a proper solution.
    // https://issuetracker.google.com/issues/309506799
    override fun AnimatedContentTransitionScope<NavBackStackEntry>.exitTransition() =
        fadeOut(snap(400))

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popEnterTransition() = null

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popExitTransition():
        ExitTransition {
        return slideOutVertically(targetOffsetY = { it })
    }
}
