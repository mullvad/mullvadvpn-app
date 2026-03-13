package net.mullvad.mullvadvpn.feature.problemreport.api

import androidx.navigation3.runtime.NavKey
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.NavResult

@Parcelize
object ProblemReportNoEmailNavKey : NavKey2

@Parcelize
object ProblemReportNoEmailConfirmedNavResult : NavResult
