package net.mullvad.mullvadvpn.compose.transitions

import android.util.Log
import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.snap
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.navigation.NavBackStackEntry
import com.ramcosta.composedestinations.spec.DestinationStyle
import com.ramcosta.composedestinations.utils.destination
import net.mullvad.mullvadvpn.compose.destinations.ConnectDestination
import net.mullvad.mullvadvpn.compose.destinations.OutOfTimeDestination
import net.mullvad.mullvadvpn.compose.destinations.WelcomeDestination

object LoginTransition : DestinationStyle.Animated {
    override fun AnimatedContentTransitionScope<NavBackStackEntry>.enterTransition() = fadeIn()

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
                    fadeOut()
                }
                else -> fadeOut(snap(400))
            }

        Log.d(
            "LoginTransition",
            "exitTransition: ${this.targetState.destination()}" + "transition: $transition"
        )
        return transition
    }

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popEnterTransition() = fadeIn()

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popExitTransition() = fadeOut()
}
