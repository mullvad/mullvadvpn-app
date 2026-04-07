package net.mullvad.mullvadvpn.feature.serveripoverride.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.common.compose.LocalSharedTransitionScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.core.scene.ListDetailSceneStrategy
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ServerIpOverrideNavKey
import net.mullvad.mullvadvpn.feature.serveripoverride.impl.ServerIpOverrides

fun EntryProviderScope<NavKey2>.serverIpOverrideEntry(navigator: Navigator) {
    entry<ServerIpOverrideNavKey>(
        metadata = ListDetailSceneStrategy.detailPane() + slideInHorizontalTransition()
    ) { navKey ->
        LocalSharedTransitionScope.current?.ServerIpOverrides(
            navArgs = navKey,
            navigator = navigator,
            animatedVisibilityScope = LocalNavAnimatedContentScope.current,
        )
    }

    resetServerIpOverrideConfirmationEntry(navigator)
    importOverrideByTextScreenEntry(navigator)
    importOverridesEntry(navigator)
    serverIpOverrideInfoEntry(navigator)
}
