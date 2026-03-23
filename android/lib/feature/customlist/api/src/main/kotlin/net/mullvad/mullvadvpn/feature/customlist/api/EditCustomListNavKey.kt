package net.mullvad.mullvadvpn.feature.customlist.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.communication.CustomListActionResultData

@Parcelize data class EditCustomListNavKey(val customListId: CustomListId) : NavKey2

@Parcelize data class EditCustomListNavResult(val value: CustomListActionResultData) : NavResult
