package net.mullvad.mullvadvpn.compose.transitions

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.navigation.NavBackStackEntry
import com.ramcosta.composedestinations.spec.DestinationStyle

object SlideInFromRightTransition : DestinationStyle.Animated {
    override fun AnimatedContentTransitionScope<NavBackStackEntry>.enterTransition() =
        slideInHorizontally(initialOffsetX = { it })

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.exitTransition() =
        slideOutHorizontally(targetOffsetX = { -it })

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popEnterTransition() =
        slideInHorizontally(initialOffsetX = { -it })

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popExitTransition() =
        slideOutHorizontally(targetOffsetX = { it })
}
