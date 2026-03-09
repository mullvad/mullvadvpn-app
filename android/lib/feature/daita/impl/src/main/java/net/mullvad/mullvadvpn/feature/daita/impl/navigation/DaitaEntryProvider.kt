package net.mullvad.mullvadvpn.feature.daita.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.common.compose.LocalSharedTransitionScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.daita.api.DaitaNavKey
import net.mullvad.mullvadvpn.feature.daita.impl.Daita

fun EntryProviderScope<NavKey2>.daitaEntry(navigator: Navigator) {
    entry<DaitaNavKey>(metadata = slideInHorizontalTransition()) { navKey ->
        LocalSharedTransitionScope.current?.Daita(
            navigator = navigator,
            isModal = navKey.isModal,
            animatedVisibilityScope = LocalNavAnimatedContentScope.current,
        )
    }

    daitaDirectOnlyConfirmationEntry(navigator)
    daitaDirectOnlyInfoEntry(navigator)
}
