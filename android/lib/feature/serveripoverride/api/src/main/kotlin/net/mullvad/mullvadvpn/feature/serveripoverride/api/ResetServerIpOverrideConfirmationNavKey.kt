package net.mullvad.mullvadvpn.feature.serveripoverride.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult

@Parcelize data object ResetServerIpOverrideConfirmationNavKey : NavKey2

@Parcelize
data class ResetServerIpOverrideConfirmationNavResult(val clearSuccessful: Boolean) : NavResult
