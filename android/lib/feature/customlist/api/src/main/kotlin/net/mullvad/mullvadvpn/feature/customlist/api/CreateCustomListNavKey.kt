package net.mullvad.mullvadvpn.feature.customlist.api

import androidx.navigation3.runtime.NavKey
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.lib.model.GeoLocationId

@Serializable data class CreateCustomListNavKey(val locationCode: GeoLocationId?) : NavKey
