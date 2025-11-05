package net.mullvad.mullvadvpn.compose.transitions

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.spring
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.navigation.NavBackStackEntry
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.constant.ENTER_TRANSITION_SLIDE_FACTOR
import net.mullvad.mullvadvpn.constant.EXIT_TRANSITION_SLIDE_FACTOR

object SlideInFromRightTransition : DestinationStyle.Animated() {
    override val enterTransition:
        AnimatedContentTransitionScope<NavBackStackEntry>.() -> EnterTransition =
        {
            fadeIn(spring()) +
            slideIntoContainer(
                towards = AnimatedContentTransitionScope.SlideDirection.Start,
                initialOffset = { (it * ENTER_TRANSITION_SLIDE_FACTOR).toInt() }
            )
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
            slideOutOfContainer(
                towards = AnimatedContentTransitionScope.SlideDirection.End,
                targetOffset = { (it * ENTER_TRANSITION_SLIDE_FACTOR).toInt() }
            ) + fadeOut(spring())
        }
}
