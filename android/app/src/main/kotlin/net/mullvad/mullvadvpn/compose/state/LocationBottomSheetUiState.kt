package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.RelayItem

sealed interface LocationBottomSheetUiState {
    val item: RelayItem
    val canBeSetAsEntry: Boolean
    val canBeSetAsExit: Boolean
    val canBeRemovedAsEntry: Boolean

    data class CustomListsEntry(
        override val item: RelayItem.Location,
        override val canBeSetAsEntry: Boolean,
        override val canBeSetAsExit: Boolean,
        override val canBeRemovedAsEntry: Boolean,
        val customListId: CustomListId,
        val customListName: CustomListName,
    ) : LocationBottomSheetUiState

    data class Location(
        override val item: RelayItem.Location,
        override val canBeSetAsEntry: Boolean,
        override val canBeSetAsExit: Boolean,
        override val canBeRemovedAsEntry: Boolean,
        val customLists: List<RelayItem.CustomList>,
    ) : LocationBottomSheetUiState

    data class CustomList(
        override val item: RelayItem.CustomList,
        override val canBeSetAsEntry: Boolean,
        override val canBeSetAsExit: Boolean,
        override val canBeRemovedAsEntry: Boolean,
    ) : LocationBottomSheetUiState
}
