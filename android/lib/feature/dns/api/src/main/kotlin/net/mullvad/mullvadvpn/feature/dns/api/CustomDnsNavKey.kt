package net.mullvad.mullvadvpn.feature.dns.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult

@Parcelize
data class CustomDnsNavKey(val index: Int? = null, val initialValue: String? = null) : NavKey2

@Parcelize
sealed interface CustomDnsNavResult : NavResult {
    data class Success(val isDnsListEmpty: Boolean) : CustomDnsNavResult

    data object Error : CustomDnsNavResult
}
