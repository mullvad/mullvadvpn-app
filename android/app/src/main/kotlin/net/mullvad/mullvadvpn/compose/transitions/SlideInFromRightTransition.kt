package net.mullvad.mullvadvpn.compose.transitions

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.snap
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.navigation.NavBackStackEntry
import com.ramcosta.composedestinations.generated.destinations.NoDaemonScreenDestination
import com.ramcosta.composedestinations.spec.DestinationStyle
import com.ramcosta.composedestinations.utils.destination
import net.mullvad.mullvadvpn.constant.SCREEN_ANIMATION_TIME_MILLIS
import net.mullvad.mullvadvpn.constant.withHorizontalScalingFactor

object SlideInFromRightTransition : DestinationStyle.Animated() {
    override val enterTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> EnterTransition =
        {
            slideInHorizontally(initialOffsetX = { it })
        }

    override val exitTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> ExitTransition =
        {
            when (targetState.destination()) {
                NoDaemonScreenDestination -> fadeOut(snap(SCREEN_ANIMATION_TIME_MILLIS))
                else -> slideOutHorizontally(targetOffsetX = { -it.withHorizontalScalingFactor() })
            }
        }

    override val popEnterTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> EnterTransition =
        {
            when (initialState.destination()) {
                NoDaemonScreenDestination -> fadeIn(snap(0))
                else -> slideInHorizontally(initialOffsetX = { -it.withHorizontalScalingFactor() })
            }
        }

    override val popExitTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> ExitTransition =
        {
            slideOutHorizontally(targetOffsetX = { it })
        }
}
