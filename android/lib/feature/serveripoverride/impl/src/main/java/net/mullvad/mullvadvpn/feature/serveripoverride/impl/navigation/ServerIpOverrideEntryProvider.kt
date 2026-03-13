package net.mullvad.mullvadvpn.feature.serveripoverride.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.common.compose.LocalSharedTransitionScope
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ServerIpOverrideNavKey
import net.mullvad.mullvadvpn.feature.serveripoverride.impl.ServerIpOverrides
import net.mullvad.mullvadvpn.feature.serveripoverride.impl.ServerIpOverridesScreen

fun EntryProviderScope<NavKey2>.serverIpOverrideEntry(navigator: Navigator) {
    entry<ServerIpOverrideNavKey> { navKey ->
        LocalSharedTransitionScope.current?.ServerIpOverrides(
            navArgs = navKey,
            navigator = navigator,
            animatedVisibilityScope = LocalNavAnimatedContentScope.current,
        )
    }

    resetServerIpOverrideConfirmationEntry(navigator)
    importOverrideByTextScreenEntry(navigator)
}
