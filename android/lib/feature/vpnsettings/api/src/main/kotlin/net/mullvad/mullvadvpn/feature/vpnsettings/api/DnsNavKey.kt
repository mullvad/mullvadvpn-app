package net.mullvad.mullvadvpn.feature.vpnsettings.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult

@Parcelize data class DnsNavKey(val index: Int? = null, val initialValue: String? = null) : NavKey2

sealed interface DnsNavResult : NavResult {
    @Parcelize data class Success(val isDnsListEmpty: Boolean) : DnsNavResult

    @Parcelize data object Error : DnsNavResult
}
