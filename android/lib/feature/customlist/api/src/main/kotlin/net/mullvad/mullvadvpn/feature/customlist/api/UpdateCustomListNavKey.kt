package net.mullvad.mullvadvpn.feature.customlist.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.communication.CustomListActionResultData

@Parcelize
data class UpdateCustomListNavKey(val customListId: CustomListId, val name: CustomListName) :
    NavKey2

@Parcelize data class UpdateCustomListNavResult(val value: CustomListActionResultData) : NavResult
