package net.mullvad.mullvadvpn.compose.transitions

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.snap
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.slideOutVertically
import androidx.navigation.NavBackStackEntry
import com.ramcosta.composedestinations.generated.destinations.NoDaemonDestination
import com.ramcosta.composedestinations.generated.destinations.SearchLocationDestination
import com.ramcosta.composedestinations.spec.DestinationStyle
import com.ramcosta.composedestinations.utils.destination
import net.mullvad.mullvadvpn.constant.SCREEN_ANIMATION_TIME_MILLIS
import net.mullvad.mullvadvpn.constant.withHorizontalScalingFactor

object SlideInFromBottomTransition : DestinationStyle.Animated() {
    override val enterTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> EnterTransition? =
        {
            slideInVertically(initialOffsetY = { it })
        }

    override val exitTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> ExitTransition? =
        {
            when (targetState.destination()) {
                NoDaemonDestination -> fadeOut(snap(SCREEN_ANIMATION_TIME_MILLIS))
                else -> fadeOut()
            }
        }

    override val popEnterTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> EnterTransition =
        {
            when (initialState.destination()) {
                NoDaemonDestination -> fadeIn(snap(0))
                else -> fadeIn()
            }
        }

    override val popExitTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> ExitTransition =
        {
            slideOutVertically(targetOffsetY = { it })
        }
}

object SelectLocationTransition : DestinationStyle.Animated() {
    override val enterTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> EnterTransition =
        {
            slideInVertically(initialOffsetY = { it })
        }

    // TODO temporary hack until we have a proper solution.
    // https://issuetracker.google.com/issues/309506799
    override val exitTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> ExitTransition =
        {
            when (targetState.destination()) {
                NoDaemonDestination -> fadeOut(snap(SCREEN_ANIMATION_TIME_MILLIS))
                SearchLocationDestination -> fadeOut()
                else -> slideOutHorizontally { -it.withHorizontalScalingFactor() }
            }
        }

    override val popEnterTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> EnterTransition =
        {
            when (initialState.destination()) {
                NoDaemonDestination -> fadeIn(snap(0))
                SearchLocationDestination -> fadeIn()
                else -> slideInHorizontally { -it.withHorizontalScalingFactor() }
            }
        }

    override val popExitTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> ExitTransition? =
        {
            slideOutVertically(targetOffsetY = { it })
        }
}
