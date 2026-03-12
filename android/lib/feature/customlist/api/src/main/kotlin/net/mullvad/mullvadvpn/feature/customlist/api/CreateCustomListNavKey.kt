package net.mullvad.mullvadvpn.feature.customlist.api

import androidx.navigation3.runtime.NavKey
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.lib.model.GeoLocationId

@Parcelize
data class CreateCustomListNavKey(val locationCode: GeoLocationId?) : NavKey2
