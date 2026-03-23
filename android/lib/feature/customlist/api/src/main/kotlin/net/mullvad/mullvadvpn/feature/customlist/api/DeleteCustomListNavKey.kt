package net.mullvad.mullvadvpn.feature.customlist.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.communication.CustomListActionResultData

@Parcelize
data class DeleteCustomListNavKey(val customListId: CustomListId, val name: CustomListName) :
    NavKey2

@Parcelize
data class DeleteCustomListNavResult(val value: CustomListActionResultData.Success.Deleted) :
    NavResult
