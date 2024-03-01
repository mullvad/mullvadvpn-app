package net.mullvad.mullvadvpn.compose.communication

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed interface CustomListResult : Parcelable {
    val reverseAction: CustomListAction

    @Parcelize
    data class ListCreated(
        val locationName: String,
        val customListName: String,
        override val reverseAction: CustomListAction
    ) : CustomListResult

    @Parcelize
    data class ListDeleted(val name: String, override val reverseAction: CustomListAction) :
        CustomListResult

    @Parcelize
    data class ListRenamed(val name: String, override val reverseAction: CustomListAction) :
        CustomListResult

    @Parcelize
    data class ListUpdated(val name: String, override val reverseAction: CustomListAction) :
        CustomListResult
}
