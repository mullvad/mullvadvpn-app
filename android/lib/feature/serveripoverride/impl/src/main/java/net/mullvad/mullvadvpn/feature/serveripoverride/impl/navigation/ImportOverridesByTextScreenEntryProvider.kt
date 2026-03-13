package net.mullvad.mullvadvpn.feature.serveripoverride.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.common.compose.LocalSharedTransitionScope
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ImportOverrideByTextNavKey
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ResetServerIpOverrideConfirmationNavKey
import net.mullvad.mullvadvpn.feature.serveripoverride.impl.ServerIpOverrides
import net.mullvad.mullvadvpn.feature.serveripoverride.impl.ServerIpOverridesScreen
import net.mullvad.mullvadvpn.feature.serveripoverride.impl.importbytext.ImportOverridesByText
import net.mullvad.mullvadvpn.feature.serveripoverride.impl.importbytext.ImportOverridesByTextScreen
import net.mullvad.mullvadvpn.feature.serveripoverride.impl.reset.ResetServerIpOverridesConfirmation

internal fun EntryProviderScope<NavKey2>.importOverrideByTextScreenEntry(navigator: Navigator) {
    entry<ImportOverrideByTextNavKey>(metadata = DialogSceneStrategy.dialog()) {
        ImportOverridesByText(navigator = navigator)
    }
}
