package net.mullvad.mullvadvpn.feature.serveripoverride.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult

@Parcelize data object ImportOverrideByTextNavKey : NavKey2

@Parcelize data class ImportOverrideByTextNavResult(val text: String) : NavResult
