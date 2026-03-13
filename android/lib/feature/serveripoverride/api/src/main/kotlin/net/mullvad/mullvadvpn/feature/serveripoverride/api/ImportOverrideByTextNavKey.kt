package net.mullvad.mullvadvpn.feature.serveripoverride.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.NavResult

@Parcelize
data object ImportOverrideByTextNavKey : NavKey2

@Parcelize
data class ImportOverrideByTextNavResult(val text: String) : NavResult
