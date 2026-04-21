package net.mullvad.mullvadvpn.feature.vpnsettings.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.common.compose.LocalSharedTransitionScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.core.scene.ListDetailSceneStrategy
import net.mullvad.mullvadvpn.feature.vpnsettings.api.VpnSettingsNavKey
import net.mullvad.mullvadvpn.feature.vpnsettings.impl.VpnSettings

fun EntryProviderScope<NavKey2>.vpnSettingsEntry(navigator: Navigator) {
    entry<VpnSettingsNavKey>(
        metadata = ListDetailSceneStrategy.listPane() + slideInHorizontalTransition()
    ) { navArgs ->
        LocalSharedTransitionScope.current?.VpnSettings(
            navArgs = navArgs,
            navigator = navigator,
            animatedVisibilityScope = LocalNavAnimatedContentScope.current,
        )
    }

    connectOnStartupInfoEntry(navigator)
    deviceIpInfoEntry(navigator)
    localNetworkSharingInfoEntry(navigator)
    ipv6InfoEntry(navigator)
    mtuEntry(navigator)
    quantumResistanceInfoEntry(navigator)
}
