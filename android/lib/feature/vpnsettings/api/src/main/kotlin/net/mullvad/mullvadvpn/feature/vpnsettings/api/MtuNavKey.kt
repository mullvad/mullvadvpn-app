package net.mullvad.mullvadvpn.feature.vpnsettings.api

import androidx.navigation3.runtime.NavKey
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.NavResult
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.model.Mtu

@Parcelize
data class MtuNavKey(val initialMtu: Mtu? = null) : NavKey2

@Parcelize
data class MtuNavResult(val complete: Boolean) : NavResult
