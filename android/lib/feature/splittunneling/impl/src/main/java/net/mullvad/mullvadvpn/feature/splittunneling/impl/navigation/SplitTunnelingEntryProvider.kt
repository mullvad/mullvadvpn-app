package net.mullvad.mullvadvpn.feature.splittunneling.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.common.compose.LocalSharedTransitionScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.core.scene.ListDetailSceneStrategy
import net.mullvad.mullvadvpn.feature.splittunneling.api.SplitTunnelingNavKey
import net.mullvad.mullvadvpn.feature.splittunneling.impl.SplitTunneling

fun EntryProviderScope<NavKey2>.splitTunnelingEntry(navigator: Navigator) {
    entry<SplitTunnelingNavKey>(
        metadata = ListDetailSceneStrategy.detailPane() + slideInHorizontalTransition()
    ) { navArgs ->
        LocalSharedTransitionScope.current?.SplitTunneling(
            isModal = navArgs.isModal,
            navigator = navigator,
            animatedVisibilityScope = LocalNavAnimatedContentScope.current,
        )
    }
    searchSplitTunnelingEntry(navigator)
}
