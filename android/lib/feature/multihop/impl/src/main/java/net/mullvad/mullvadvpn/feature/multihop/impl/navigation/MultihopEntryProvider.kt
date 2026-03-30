package net.mullvad.mullvadvpn.feature.multihop.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.common.compose.LocalSharedTransitionScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.core.scene.ListDetailSceneStrategy
import net.mullvad.mullvadvpn.feature.multihop.api.MultihopNavKey
import net.mullvad.mullvadvpn.feature.multihop.impl.Multihop

fun EntryProviderScope<NavKey2>.multihopEntry(navigator: Navigator) {
    entry<MultihopNavKey>(
        metadata = ListDetailSceneStrategy.detailPane() + slideInHorizontalTransition()
    ) { navKey ->
        LocalSharedTransitionScope.current?.Multihop(
            isModal = navKey.isModal,
            navigator = navigator,
            animatedVisibilityScope = LocalNavAnimatedContentScope.current,
        )
    }
}
