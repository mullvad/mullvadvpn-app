package net.mullvad.mullvadvpn.compose.transitions

import android.util.Log
import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.snap
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideOutHorizontally
import androidx.navigation.NavBackStackEntry
import com.ramcosta.composedestinations.spec.DestinationStyle
import com.ramcosta.composedestinations.utils.destination
import net.mullvad.mullvadvpn.compose.destinations.ConnectDestination
import net.mullvad.mullvadvpn.compose.destinations.OutOfTimeDestination
import net.mullvad.mullvadvpn.compose.destinations.WelcomeDestination

object LoginTransition : DestinationStyle.Animated {
    override fun AnimatedContentTransitionScope<NavBackStackEntry>.enterTransition() =
        EnterTransition.None

    // TODO temporary hack until we have a proper solution.
    // https://issuetracker.google.com/issues/309506799
    override fun AnimatedContentTransitionScope<NavBackStackEntry>.exitTransition():
        ExitTransition {
        val transition =
            when (this.targetState.destination()) {
                is OutOfTimeDestination,
                is WelcomeDestination,
                is ConnectDestination -> {
                    Log.d("LoginTransition", "was slide anim!")
                    slideOutHorizontally(targetOffsetX = { -it })
                }
                else -> fadeOut(snap(700))
            }

        Log.d(
            "LoginTransition",
            "exitTransition: ${this.targetState.destination()}" + "transition: $transition"
        )
        return transition
    }

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popEnterTransition() =
        EnterTransition.None

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popExitTransition() =
        ExitTransition.None
}
