package net.mullvad.mullvadvpn.feature.problemreport.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.problemreport.api.ProblemReportNavKey
import net.mullvad.mullvadvpn.feature.problemreport.impl.ReportProblem

fun EntryProviderScope<NavKey2>.problemReportEntry(navigator: Navigator) {
    entry<ProblemReportNavKey> {
        ReportProblem(navigator = navigator)
    }

    problemReportNoEmailEntry(navigator)
    viewLogsReportEntry(navigator)
}
