package net.mullvad.mullvadvpn.feature.anticensorship.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.common.compose.LocalSharedTransitionScope
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.anticensorship.api.AnticensorshipNavKey
import net.mullvad.mullvadvpn.feature.anticensorship.impl.AntiCensorshipSettings

fun EntryProviderScope<NavKey2>.anticensorshipEntry(navigator: Navigator) {
    entry<AnticensorshipNavKey> { navKey ->
        LocalSharedTransitionScope.current?.AntiCensorshipSettings(
            navigator = navigator,
            navArgs = navKey,
            animatedVisibilityScope = LocalNavAnimatedContentScope.current,
        )
    }
}
