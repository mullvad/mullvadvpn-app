package net.mullvad.mullvadvpn.feature.customvpn.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.core.scene.SingleOverlaySceneStrategy
import net.mullvad.mullvadvpn.feature.customvpn.api.CreateCustomTunnelNavKey
import net.mullvad.mullvadvpn.feature.customvpn.api.CustomTunnelsListNavKey
import net.mullvad.mullvadvpn.feature.customvpn.api.EditCustomTunnelNavKey
import net.mullvad.mullvadvpn.feature.customvpn.api.RemoveCustomTunnelConfigConfirmationNavKey
import net.mullvad.mullvadvpn.feature.customvpn.impl.bottomsheet.AddCustomTunnelBottomSheet
import net.mullvad.mullvadvpn.feature.customvpn.impl.confirmation.RemoveCustomTunnelConfigConfirmDialog
import net.mullvad.mullvadvpn.feature.customvpn.impl.edit.CustomTunnelEdit
import net.mullvad.mullvadvpn.feature.customvpn.impl.list.CustomTunnelsList
import net.mullvad.mullvadvpn.lib.common.compose.LocalSharedTransitionScope

fun EntryProviderScope<NavKey2>.customTunnelsEntry(navigator: Navigator) {
    entry<CustomTunnelsListNavKey>(metadata = slideInHorizontalTransition()) { navKey ->
        LocalSharedTransitionScope.current?.CustomTunnelsList(
            navigator = navigator,
            isModal = navKey.isModal,
            animatedVisibilityScope = LocalNavAnimatedContentScope.current,
        )
    }
    entry<EditCustomTunnelNavKey> { navKey ->
        CustomTunnelEdit(
            isNewConfig = navKey.new,
            navigator = navigator,
        )
    }
    entry<CreateCustomTunnelNavKey>(metadata = SingleOverlaySceneStrategy.overlay()) {
        AddCustomTunnelBottomSheet(navigator = navigator)
    }
    entry<RemoveCustomTunnelConfigConfirmationNavKey>(metadata = DialogSceneStrategy.dialog()) {
        navKey ->
        RemoveCustomTunnelConfigConfirmDialog(navigator = navigator, title = navKey.name)
    }
}
