package net.mullvad.mullvadvpn.compose.transitions

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.snap
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.navigation.NavBackStackEntry
import com.ramcosta.composedestinations.spec.DestinationStyle
import com.ramcosta.composedestinations.utils.destination
import net.mullvad.mullvadvpn.compose.destinations.LoginDestination

// This is used for OutOfTime, Welcome, and Connect destinations.
object HomeTransition : DestinationStyle.Animated {
    override fun AnimatedContentTransitionScope<NavBackStackEntry>.enterTransition() =
        when (this.initialState.destination()) {
            is LoginDestination -> slideInHorizontally(initialOffsetX = { it })
            else -> EnterTransition.None
        }

    // TODO temporary hack until we have a proper solution.
    // https://issuetracker.google.com/issues/309506799
    override fun AnimatedContentTransitionScope<NavBackStackEntry>.exitTransition() =
        fadeOut(snap(700))

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popEnterTransition() =
        EnterTransition.None

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popExitTransition() =
        ExitTransition.None
}
