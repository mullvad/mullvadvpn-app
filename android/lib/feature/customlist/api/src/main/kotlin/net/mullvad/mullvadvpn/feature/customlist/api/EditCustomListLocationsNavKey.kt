package net.mullvad.mullvadvpn.feature.customlist.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.communication.CustomListActionResultData

@Parcelize
data class EditCustomListLocationsNavKey(val customListId: CustomListId, val newList: Boolean) :
    NavKey2

@Parcelize
data class EditCustomListLocationsNavResult(val value: CustomListActionResultData.Success.Renamed) :
    NavResult
