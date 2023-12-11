package net.mullvad.mullvadvpn.compose.transitions

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.core.snap
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.navigation.NavBackStackEntry
import com.ramcosta.composedestinations.spec.DestinationStyle
import com.ramcosta.composedestinations.utils.destination
import net.mullvad.mullvadvpn.compose.destinations.LoginDestination
import net.mullvad.mullvadvpn.constant.SCREEN_ANIMATION_TIME_MILLIS

// This is used for OutOfTime, Welcome, and Connect destinations.
object HomeTransition : DestinationStyle.Animated {
    override fun AnimatedContentTransitionScope<NavBackStackEntry>.enterTransition() =
        when (this.initialState.destination()) {
            is LoginDestination -> fadeIn()
            else -> EnterTransition.None
        }

    // TODO temporary hack until we have a proper solution.
    // https://issuetracker.google.com/issues/309506799
    override fun AnimatedContentTransitionScope<NavBackStackEntry>.exitTransition() =
        fadeOut(snap(SCREEN_ANIMATION_TIME_MILLIS))

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popEnterTransition() =
        EnterTransition.None

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popExitTransition() = fadeOut()
}
