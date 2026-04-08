package net.mullvad.mullvadvpn.feature.language.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.core.scene.ListDetailSceneStrategy
import net.mullvad.mullvadvpn.feature.language.api.LanguageNavKey
import net.mullvad.mullvadvpn.feature.language.impl.Language

fun EntryProviderScope<NavKey2>.languageEntry(navigator: Navigator) {
    entry<LanguageNavKey>(
        metadata = ListDetailSceneStrategy.detailPane() + slideInHorizontalTransition()
    ) {
        Language(navigator = navigator)
    }
}

