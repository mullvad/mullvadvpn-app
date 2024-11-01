package net.mullvad.mullvadvpn.compose.transitions

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.fadeIn
import androidx.navigation.NavBackStackEntry
import com.ramcosta.composedestinations.generated.destinations.LoginDestination
import com.ramcosta.composedestinations.spec.DestinationStyle
import com.ramcosta.composedestinations.utils.destination

// This is used for OutOfTime, Welcome, and Connect destinations.
object HomeTransition : DestinationStyle.Animated() {

    override val enterTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> EnterTransition =
        {
            when (initialState.destination()) {
                LoginDestination -> fadeIn()
                else -> EnterTransition.None
            }
        }

    override val exitTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> ExitTransition =
        {
            ExitTransition.None
        }

    override val popEnterTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> EnterTransition =
        {
            EnterTransition.None
        }

    override val popExitTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> ExitTransition =
        {
            ExitTransition.None
        }
}
