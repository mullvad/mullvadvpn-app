package net.mullvad.mullvadvpn.feature.problemreport.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.problemreport.api.ViewLogsNavKey
import net.mullvad.mullvadvpn.feature.problemreport.impl.viewlogs.ViewLogs

internal fun EntryProviderScope<NavKey2>.viewLogsReportEntry(navigator: Navigator) {
    entry<ViewLogsNavKey>(metadata = slideInHorizontalTransition()) {
        ViewLogs(navigator = navigator)
    }
}
