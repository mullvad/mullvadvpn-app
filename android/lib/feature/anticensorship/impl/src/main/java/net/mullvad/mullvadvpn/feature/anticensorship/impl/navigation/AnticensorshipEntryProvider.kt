package net.mullvad.mullvadvpn.feature.anticensorship.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.common.compose.LocalSharedTransitionScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.core.scene.ListDetailSceneStrategy
import net.mullvad.mullvadvpn.feature.anticensorship.api.AntiCensorshipNavKey
import net.mullvad.mullvadvpn.feature.anticensorship.impl.AntiCensorshipSettings

fun EntryProviderScope<NavKey2>.anticensorshipEntry(navigator: Navigator) {
    entry<AntiCensorshipNavKey>(
        metadata = ListDetailSceneStrategy.detailPane() + slideInHorizontalTransition()
    ) { navKey ->
        LocalSharedTransitionScope.current?.AntiCensorshipSettings(
            navigator = navigator,
            navArgs = navKey,
            animatedVisibilityScope = LocalNavAnimatedContentScope.current,
        )
    }

    customPortEntry(navigator)
    selectPortEntry(navigator)
}
