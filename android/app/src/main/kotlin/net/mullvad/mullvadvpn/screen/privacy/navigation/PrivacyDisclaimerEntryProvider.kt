package net.mullvad.mullvadvpn.screen.privacy.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.core.nav3.PrivacyDisclaimerNavKey
import net.mullvad.mullvadvpn.screen.privacy.PrivacyDisclaimer
import net.mullvad.mullvadvpn.screen.splash.Splash

fun EntryProviderScope<NavKey2>.privacyDisclaimerEntry(navigator: Navigator) {
    entry<PrivacyDisclaimerNavKey> {
        PrivacyDisclaimer(navigator = navigator)
    }
}
