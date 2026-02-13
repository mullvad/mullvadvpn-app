package net.mullvad.mullvadvpn.core.animation

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.spring
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.navigation.NavBackStackEntry
import com.ramcosta.composedestinations.spec.DestinationStyle

object LoginTransition : DestinationStyle.Animated() {
    override val enterTransition:
        (AnimatedContentTransitionScope<NavBackStackEntry>.() -> EnterTransition) =
        {
            fadeIn(spring())
        }

    override val exitTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> ExitTransition =
        {
            // TODO How to solve this?
            fadeOut(spring())
            // when (this.targetState.destination()) {
            // is OutOfTimeDestination,
            // is WelcomeDestination,
            // is ConnectDestination,
            // is DeviceListDestination -> fadeOut(spring())
            // else -> ExitTransition.None
            // }
        }

    override val popEnterTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> EnterTransition =
        {
            fadeIn(spring())
        }

    override val popExitTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> ExitTransition =
        {
            fadeOut(spring())
        }
}
