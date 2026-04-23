package net.mullvad.mullvadvpn.lib.feature.customvpn.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.core.scene.SingleOverlaySceneStrategy
import net.mullvad.mullvadvpn.feature.customvpn.api.CreateCustomTunnelNavKey
import net.mullvad.mullvadvpn.feature.customvpn.api.CustomTunnelsListNavKey
import net.mullvad.mullvadvpn.feature.customvpn.api.EditCustomTunnelNavKey
import net.mullvad.mullvadvpn.lib.common.compose.LocalSharedTransitionScope
import net.mullvad.mullvadvpn.lib.feature.customvpn.impl.bottomsheet.AddCustomTunnelBottomSheet
import net.mullvad.mullvadvpn.lib.feature.customvpn.impl.list.CustomTunnelsList
import net.mullvad.mullvadvpn.lib.feature.impl.PersonalVpn

fun EntryProviderScope<NavKey2>.customTunnelsEntry(navigator: Navigator) {
    entry<CustomTunnelsListNavKey>(metadata = slideInHorizontalTransition()) { navKey ->
        LocalSharedTransitionScope.current?.CustomTunnelsList(
            navigator = navigator,
            isModal = navKey.isModal,
            animatedVisibilityScope = LocalNavAnimatedContentScope.current,
        )
    }
    entry<EditCustomTunnelNavKey> { navKey ->
        LocalSharedTransitionScope.current?.PersonalVpn(
            isModal = navKey.new,
            navigator = navigator,
            animatedVisibilityScope = LocalNavAnimatedContentScope.current,
        )
    }
    entry<CreateCustomTunnelNavKey>(metadata = SingleOverlaySceneStrategy.overlay()) {
        AddCustomTunnelBottomSheet(navigator = navigator)
    }
}


