package net.mullvad.mullvadvpn.compose.transitions

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.core.snap
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.navigation.NavBackStackEntry
import com.ramcosta.composedestinations.spec.DestinationStyle
import com.ramcosta.composedestinations.utils.destination
import net.mullvad.mullvadvpn.compose.destinations.NoDaemonScreenDestination
import net.mullvad.mullvadvpn.constant.SCREEN_ANIMATION_TIME_MILLIS

object SlideInFromRightLeafTransition : DestinationStyle.Animated {
    override fun AnimatedContentTransitionScope<NavBackStackEntry>.enterTransition() =
        slideInHorizontally(initialOffsetX = { it })

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.exitTransition() =
        when (targetState.destination()) {
            NoDaemonScreenDestination -> fadeOut(snap(SCREEN_ANIMATION_TIME_MILLIS))
            else -> fadeOut()
        }

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popEnterTransition() =
        fadeIn(snap(0))

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popExitTransition() =
        slideOutHorizontally(targetOffsetX = { it })
}
