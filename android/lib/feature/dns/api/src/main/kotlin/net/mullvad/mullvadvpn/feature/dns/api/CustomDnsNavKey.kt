package net.mullvad.mullvadvpn.feature.dns.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult

@Parcelize
data class CustomDnsNavKey(val index: Int? = null, val initialValue: String? = null) : NavKey2

sealed interface CustomDnsNavResult : NavResult {
    @Parcelize data class Success(val isDnsListEmpty: Boolean) : CustomDnsNavResult

    @Parcelize data object Error : CustomDnsNavResult
}
