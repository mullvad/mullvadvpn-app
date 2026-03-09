package net.mullvad.mullvadvpn.screen.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.screen.privacy.PrivacyDisclaimer

fun EntryProviderScope<NavKey2>.privacyDisclaimerEntry(navigator: Navigator) {
    entry<PrivacyDisclaimerNavKey> { PrivacyDisclaimer(navigator = navigator) }
}
