package net.mullvad.mullvadvpn.compose.transitions

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.core.snap
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.slideOutVertically
import androidx.navigation.NavBackStackEntry
import com.ramcosta.composedestinations.spec.DestinationStyle
import com.ramcosta.composedestinations.utils.destination
import net.mullvad.mullvadvpn.compose.destinations.NoDaemonScreenDestination

object SettingsTransition : DestinationStyle.Animated {
    override fun AnimatedContentTransitionScope<NavBackStackEntry>.enterTransition() =
        slideInVertically(initialOffsetY = { it })

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.exitTransition() =
        when (targetState.destination()) {
            NoDaemonScreenDestination -> fadeOut(snap(400))
            else -> slideOutHorizontally(targetOffsetX = { -it/3 })
        }

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popEnterTransition() =
        when (initialState.destination()) {
            NoDaemonScreenDestination -> fadeIn(snap(0))
            else -> slideInHorizontally(initialOffsetX = { -it/3 })
        }

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popExitTransition() =
        slideOutVertically(targetOffsetY = { it })
}
