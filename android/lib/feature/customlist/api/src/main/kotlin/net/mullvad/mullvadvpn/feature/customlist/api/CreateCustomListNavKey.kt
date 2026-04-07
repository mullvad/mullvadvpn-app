package net.mullvad.mullvadvpn.feature.customlist.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.communication.CustomListActionResultData

@Parcelize data class CreateCustomListNavKey(val locationCode: GeoLocationId? = null) : NavKey2

@Parcelize
data class CreateCustomListNavResult(
    val value: CustomListActionResultData.Success.CreatedWithLocations
) : NavResult
