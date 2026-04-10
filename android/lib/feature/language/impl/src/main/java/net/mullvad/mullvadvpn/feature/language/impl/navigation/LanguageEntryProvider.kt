package net.mullvad.mullvadvpn.feature.language.impl.navigation

import androidx.annotation.RequiresApi
import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.language.api.LanguageNavKey
import net.mullvad.mullvadvpn.feature.language.impl.Language

@RequiresApi(android.os.Build.VERSION_CODES.TIRAMISU)
fun EntryProviderScope<NavKey2>.languageEntry(navigator: Navigator) {
    entry<LanguageNavKey>(metadata = slideInHorizontalTransition()) {
        Language(navigator = navigator)
    }
}
