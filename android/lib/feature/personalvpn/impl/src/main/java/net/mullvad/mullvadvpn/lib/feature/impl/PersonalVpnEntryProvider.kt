package net.mullvad.mullvadvpn.lib.feature.impl

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.common.compose.LocalSharedTransitionScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.personalvpn.api.PersonalVpnNavKey

fun EntryProviderScope<NavKey2>.personalVpnEntry(navigator: Navigator) {
    entry<PersonalVpnNavKey>(metadata = slideInHorizontalTransition()) { navKey ->
        LocalSharedTransitionScope.current?.PersonalVpn(
            isModal = navKey.isModal,
            navigator = navigator,
            animatedVisibilityScope = LocalNavAnimatedContentScope.current,
        )
    }
}
