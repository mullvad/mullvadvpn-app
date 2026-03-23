package net.mullvad.mullvadvpn.feature.problemreport.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.problemreport.api.ProblemReportNoEmailNavKey
import net.mullvad.mullvadvpn.feature.problemreport.impl.noemail.ReportProblemNoEmail

internal fun EntryProviderScope<NavKey2>.problemReportNoEmailEntry(navigator: Navigator) {
    entry<ProblemReportNoEmailNavKey>(metadata = DialogSceneStrategy.dialog()) {
        ReportProblemNoEmail(navigator = navigator)
    }
}
