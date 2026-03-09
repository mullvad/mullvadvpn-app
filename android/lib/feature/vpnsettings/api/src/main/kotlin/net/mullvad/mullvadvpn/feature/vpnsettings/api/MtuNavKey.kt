package net.mullvad.mullvadvpn.feature.vpnsettings.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult
import net.mullvad.mullvadvpn.lib.model.Mtu

@Parcelize data class MtuNavKey(val initialMtu: Mtu? = null) : NavKey2

@Parcelize data class MtuNavResult(val complete: Boolean) : NavResult
