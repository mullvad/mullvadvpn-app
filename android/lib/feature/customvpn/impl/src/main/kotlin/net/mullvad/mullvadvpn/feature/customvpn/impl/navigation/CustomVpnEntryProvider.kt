package net.mullvad.mullvadvpn.feature.customvpn.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.core.scene.ListDetailSceneStrategy
import net.mullvad.mullvadvpn.core.scene.SingleOverlaySceneStrategy
import net.mullvad.mullvadvpn.feature.customvpn.api.CreateCustomVpnNavKey
import net.mullvad.mullvadvpn.feature.customvpn.api.CustomVpnListNavKey
import net.mullvad.mullvadvpn.feature.customvpn.api.EditCustomTunnelNavKey
import net.mullvad.mullvadvpn.feature.customvpn.api.RemoveCustomTunnelConfigConfirmationNavKey
import net.mullvad.mullvadvpn.feature.customvpn.impl.bottomsheet.AddCustomTunnelBottomSheet
import net.mullvad.mullvadvpn.feature.customvpn.impl.confirmation.RemoveCustomTunnelConfigConfirmDialog
import net.mullvad.mullvadvpn.feature.customvpn.impl.edit.CustomVpnEdit
import net.mullvad.mullvadvpn.feature.customvpn.impl.list.CustomVpnList
import net.mullvad.mullvadvpn.lib.common.compose.LocalSharedTransitionScope

fun EntryProviderScope<NavKey2>.customVpnEntry(navigator: Navigator) {
    entry<CustomVpnListNavKey>(
        metadata = ListDetailSceneStrategy.detailPane() + slideInHorizontalTransition(),
    ) { navKey ->
        LocalSharedTransitionScope.current?.CustomVpnList(
            navigator = navigator,
            isModal = navKey.isModal,
            animatedVisibilityScope = LocalNavAnimatedContentScope.current,
        )
    }
    entry<EditCustomTunnelNavKey>(metadata = slideInHorizontalTransition()) { navKey ->
        CustomVpnEdit(
            isNewConfig = navKey.new,
            navigator = navigator,
        )
    }
    entry<CreateCustomVpnNavKey>(metadata = SingleOverlaySceneStrategy.overlay()) {
        AddCustomTunnelBottomSheet(navigator = navigator)
    }
    entry<RemoveCustomTunnelConfigConfirmationNavKey>(metadata = DialogSceneStrategy.dialog()) {
        navKey ->
        RemoveCustomTunnelConfigConfirmDialog(navigator = navigator, title = navKey.name)
    }
}
