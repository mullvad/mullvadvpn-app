package net.mullvad.mullvadvpn.feature.customvpn.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult

@Parcelize data class RemoveCustomTunnelConfigConfirmationNavKey(val name: String) : NavKey2

@Parcelize data object RemoveCustomTunnelConfigConfirmationNavResult : NavResult
