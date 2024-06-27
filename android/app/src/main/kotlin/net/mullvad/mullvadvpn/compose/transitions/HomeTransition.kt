package net.mullvad.mullvadvpn.compose.transitions

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.snap
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.navigation.NavBackStackEntry
import com.ramcosta.composedestinations.generated.destinations.LoginDestination
import com.ramcosta.composedestinations.spec.DestinationStyle
import com.ramcosta.composedestinations.utils.destination
import net.mullvad.mullvadvpn.constant.SCREEN_ANIMATION_TIME_MILLIS

// This is used for OutOfTime, Welcome, and Connect destinations.
object HomeTransition : DestinationStyle.Animated() {

    override val enterTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> EnterTransition =
        {
            when (this.initialState.destination()) {
                is LoginDestination -> fadeIn()
                else -> EnterTransition.None
            }
        }

    // TODO temporary hack until we have a proper solution.
    // https://issuetracker.google.com/issues/309506799
    override val exitTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> ExitTransition =
        {
            fadeOut(snap(SCREEN_ANIMATION_TIME_MILLIS))
        }

    override val popEnterTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> EnterTransition =
        {
            EnterTransition.None
        }

    override val popExitTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> ExitTransition =
        {
            fadeOut()
        }
}
