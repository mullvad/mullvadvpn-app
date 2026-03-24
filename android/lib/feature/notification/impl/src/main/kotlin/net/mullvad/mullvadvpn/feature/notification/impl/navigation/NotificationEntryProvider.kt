package net.mullvad.mullvadvpn.feature.notification.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.core.scene.ListDetailSceneStrategy
import net.mullvad.mullvadvpn.feature.notification.api.NotificationSettingsNavKey
import net.mullvad.mullvadvpn.feature.notification.impl.NotificationSettings

fun EntryProviderScope<NavKey2>.notificationEntry(navigator: Navigator) {
    entry<NotificationSettingsNavKey>(
        metadata = ListDetailSceneStrategy.detailPane() + slideInHorizontalTransition()
    ) {
        NotificationSettings(navigator = navigator)
    }
}
