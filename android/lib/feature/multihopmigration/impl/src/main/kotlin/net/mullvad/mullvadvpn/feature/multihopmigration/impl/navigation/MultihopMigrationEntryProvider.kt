package net.mullvad.mullvadvpn.feature.multihopmigration.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.multihopmigration.api.MultihopMigrationNavKey
import net.mullvad.mullvadvpn.feature.multihopmigration.impl.MultihopMigration

fun EntryProviderScope<NavKey2>.multihopMigrationEntry(navigator: Navigator) {
    entry<MultihopMigrationNavKey>(metadata = slideInHorizontalTransition()) { navKey ->
        MultihopMigration(navKey = navKey, navigator = navigator)
    }
}
