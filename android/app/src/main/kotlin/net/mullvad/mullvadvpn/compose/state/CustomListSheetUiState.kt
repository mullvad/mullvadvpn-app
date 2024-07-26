package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName

data class CustomListSheetUiState(
    val customListId: CustomListId,
    val customListName: CustomListName
)
