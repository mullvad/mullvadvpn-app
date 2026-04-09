package net.mullvad.mullvadvpn.feature.serveripoverride.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult

@Parcelize data class ImportOverridesNavKey(val overridesActive: Boolean) : NavKey2

@Parcelize data object ImportOverrideByFileNavResult : NavResult
