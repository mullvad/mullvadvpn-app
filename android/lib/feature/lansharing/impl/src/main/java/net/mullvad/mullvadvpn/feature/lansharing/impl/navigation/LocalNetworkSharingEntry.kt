package net.mullvad.mullvadvpn.feature.lansharing.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.core.scene.ListDetailSceneStrategy
import net.mullvad.mullvadvpn.feature.lansharing.api.LocalNetworkSharingNavKey
import net.mullvad.mullvadvpn.feature.lansharing.impl.LocalNetworkSharing
import net.mullvad.mullvadvpn.lib.common.compose.LocalSharedTransitionScope

fun EntryProviderScope<NavKey2>.localNetworkSharingEntry(navigator: Navigator) {
    entry<LocalNetworkSharingNavKey>(
        metadata = ListDetailSceneStrategy.detailPane() + slideInHorizontalTransition()
    ) { navKey ->
        LocalSharedTransitionScope.current?.LocalNetworkSharing(
            navigator = navigator,
            isModal = navKey.isModal,
            animatedVisibilityScope = LocalNavAnimatedContentScope.current,
        )
    }
}
