package net.mullvad.mullvadvpn.feature.problemreport.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.core.scene.ListDetailSceneStrategy
import net.mullvad.mullvadvpn.feature.problemreport.api.ProblemReportNavKey
import net.mullvad.mullvadvpn.feature.problemreport.impl.ReportProblem

fun EntryProviderScope<NavKey2>.problemReportEntry(navigator: Navigator) {
    entry<ProblemReportNavKey>(
        metadata = ListDetailSceneStrategy.detailPane() + slideInHorizontalTransition()
    ) {
        ReportProblem(navigator = navigator)
    }

    problemReportNoEmailEntry(navigator)
    viewLogsReportEntry(navigator)
}
