package net.mullvad.mullvadvpn.feature.multihop.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.common.compose.LocalSharedTransitionScope
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.multihop.api.MultihopNavKey
import net.mullvad.mullvadvpn.feature.multihop.impl.Multihop
import net.mullvad.mullvadvpn.feature.multihop.impl.MultihopScreen

fun EntryProviderScope<NavKey2>.multihopEntry(navigator: Navigator) {
    entry<MultihopNavKey> { navKey ->
        LocalSharedTransitionScope.current?.Multihop(
            isModal = navKey.isModal,
            navigator = navigator,
            animatedVisibilityScope = LocalNavAnimatedContentScope.current,
        )
    }
}
