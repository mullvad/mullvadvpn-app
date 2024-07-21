package net.mullvad.mullvadvpn.compose.communication

import android.os.Parcelable
import kotlinx.parcelize.IgnoredOnParcel
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.lib.model.CustomListName

sealed interface CustomListActionResultData : Parcelable {
    val undo: CustomListAction?

    @Parcelize
    data class CreatedWithLocations(
        val customListName: CustomListName,
        val locationNames: List<String>,
        override val undo: CustomListAction
    ) : CustomListActionResultData

    @Parcelize
    data class Deleted(
        val customListName: CustomListName,
        override val undo: CustomListAction.Create
    ) : CustomListActionResultData

    @Parcelize
    data class Renamed(val newName: CustomListName, override val undo: CustomListAction) :
        CustomListActionResultData

    @Parcelize
    data class LocationAdded(
        val customListName: CustomListName,
        val locationName: String,
        override val undo: CustomListAction
    ) : CustomListActionResultData

    @Parcelize
    data class LocationRemoved(
        val customListName: CustomListName,
        val locationName: String,
        override val undo: CustomListAction
    ) : CustomListActionResultData

    @Parcelize
    data class LocationChanged(
        val customListName: CustomListName,
        override val undo: CustomListAction
    ) : CustomListActionResultData

    @Parcelize
    data object GenericError : CustomListActionResultData {
        @IgnoredOnParcel override val undo: CustomListAction? = null
    }

    fun hasUndo() = undo != null
}
