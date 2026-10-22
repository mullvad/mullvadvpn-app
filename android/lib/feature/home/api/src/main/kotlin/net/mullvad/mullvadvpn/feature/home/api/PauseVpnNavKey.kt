package net.mullvad.mullvadvpn.feature.home.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult

@Parcelize object PauseVpnNavKey : NavKey2

@Parcelize data class PauseVpnNavResult(val seconds: Long) : NavResult
