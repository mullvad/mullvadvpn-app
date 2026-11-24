package net.mullvad.mullvadvpn.feature.customvpn.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult

@Parcelize data class EditCustomTunnelNavKey(val new: Boolean) : NavKey2

@Parcelize data object CustomTunnelConfigDeletedResult : NavResult

@Parcelize data object CustomTunnelConfigSavedResult : NavResult
