package net.mullvad.mullvadvpn.feature.dns.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.common.compose.LocalSharedTransitionScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.core.scene.ListDetailSceneStrategy
import net.mullvad.mullvadvpn.feature.dns.api.DnsSettingsNavKey
import net.mullvad.mullvadvpn.feature.dns.impl.DnsSettings

fun EntryProviderScope<NavKey2>.dnsSettingsEntry(navigator: Navigator) {
    entry<DnsSettingsNavKey>(
        metadata = ListDetailSceneStrategy.detailPane() + slideInHorizontalTransition()
    ) { navKey ->
        LocalSharedTransitionScope.current?.DnsSettings(
            navigator = navigator,
            navArgs = navKey,
            animatedVisibilityScope = LocalNavAnimatedContentScope.current,
        )
    }
    customDnsInfoEntry(navigator)
    customDnsEntry(navigator)
    malwareInfoEntry(navigator)
    contentBlockersInfoEntry(navigator)
}
