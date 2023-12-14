package net.mullvad.mullvadvpn.compose.transitions

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.navigation.NavBackStackEntry
import com.ramcosta.composedestinations.spec.DestinationStyle

object DefaultTransition : DestinationStyle.Animated {
    override fun AnimatedContentTransitionScope<NavBackStackEntry>.enterTransition() = fadeIn()

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.exitTransition() = fadeOut()

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popEnterTransition() = fadeIn()

    override fun AnimatedContentTransitionScope<NavBackStackEntry>.popExitTransition() = fadeOut()
}
