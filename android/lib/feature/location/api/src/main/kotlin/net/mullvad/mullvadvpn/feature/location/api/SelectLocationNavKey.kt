package net.mullvad.mullvadvpn.feature.location.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult

@Parcelize object SelectLocationNavKey : NavKey2

@Parcelize data class SelectLocationNavResult(val connect: Boolean) : NavResult
