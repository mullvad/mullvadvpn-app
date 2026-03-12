package net.mullvad.mullvadvpn.feature.daita.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.common.compose.LocalSharedTransitionScope
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.daita.api.DaitaNavKey
import net.mullvad.mullvadvpn.feature.daita.impl.Daita
import net.mullvad.mullvadvpn.feature.daita.impl.DaitaScreen

fun EntryProviderScope<NavKey2>.daitaEntry(navigator: Navigator) {
    entry<DaitaNavKey> { navKey ->
        LocalSharedTransitionScope.current?.Daita(
            navigator = navigator,
            isModal = navKey.isModal,
            animatedVisibilityScope = LocalNavAnimatedContentScope.current,
        )
    }
}
