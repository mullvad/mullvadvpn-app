package net.mullvad.mullvadvpn.feature.serveripoverride.api

import androidx.navigation3.runtime.NavKey
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.NavResult

@Parcelize
data object ResetServerIpOverrideConfirmationNavKey : NavKey2

@Parcelize
data class ResetServerIpOverrideConfirmationNavResult(val clearSuccessful: Boolean) : NavResult
