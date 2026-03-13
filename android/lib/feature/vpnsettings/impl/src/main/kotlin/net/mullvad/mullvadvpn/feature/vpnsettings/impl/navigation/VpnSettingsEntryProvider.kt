package net.mullvad.mullvadvpn.feature.vpnsettings.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.common.compose.LocalSharedTransitionScope
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.vpnsettings.api.VpnSettingsNavKey
import net.mullvad.mullvadvpn.feature.vpnsettings.impl.VpnSettings
import net.mullvad.mullvadvpn.feature.vpnsettings.impl.VpnSettingsScreen

fun EntryProviderScope<NavKey2>.vpnSettingsEntry(navigator: Navigator) {
    entry<VpnSettingsNavKey> { navArgs ->
        LocalSharedTransitionScope.current?.VpnSettings(
            navArgs = navArgs,
            navigator = navigator,
            animatedVisibilityScope = LocalNavAnimatedContentScope.current,
        )
    }

    connectOnStartupInfoEntry(navigator)
    contentBlockersInfoEntry(navigator)
    customDnsInfoEntry(navigator)
    deviceIpInfoEntry(navigator)
    ipv6InfoEntry(navigator)
    malwareInfoEntry(navigator)
    mtuEntry(navigator)
    quantumResistanceInfoEntry(navigator)
}
