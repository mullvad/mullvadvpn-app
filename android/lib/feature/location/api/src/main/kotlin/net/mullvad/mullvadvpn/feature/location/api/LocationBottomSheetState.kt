package net.mullvad.mullvadvpn.feature.location.api

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayListType

@Parcelize
sealed interface LocationBottomSheetState : Parcelable {
    val item: RelayItem
    val relayListType: RelayListType

    data class ShowCustomListsEntryBottomSheet(
        val customListId: CustomListId,
        override val item: RelayItem.Location,
        override val relayListType: RelayListType,
    ) : LocationBottomSheetState

    data class ShowLocationBottomSheet(
        override val item: RelayItem.Location,
        override val relayListType: RelayListType,
    ) : LocationBottomSheetState

    data class ShowEditCustomListBottomSheet(
        override val item: RelayItem.CustomList,
        override val relayListType: RelayListType,
    ) : LocationBottomSheetState
}
