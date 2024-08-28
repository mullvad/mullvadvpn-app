package net.mullvad.mullvadvpn.compose.communication

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.lib.model.CustomListName

sealed interface CustomListActionResultData : Parcelable {

    sealed interface Success : CustomListActionResultData, Parcelable {
        val undo: CustomListAction

        @Parcelize
        data class CreatedWithLocations(
            val customListName: CustomListName,
            val locationNames: List<String>,
            override val undo: CustomListAction,
        ) : Success

        @Parcelize
        data class Deleted(
            val customListName: CustomListName,
            override val undo: CustomListAction.Create,
        ) : Success

        @Parcelize
        data class Renamed(
            val newName: CustomListName,
            override val undo: CustomListAction.Rename,
        ) : Success

        @Parcelize
        data class LocationAdded(
            val customListName: CustomListName,
            val locationName: String,
            override val undo: CustomListAction,
        ) : Success

        @Parcelize
        data class LocationRemoved(
            val customListName: CustomListName,
            val locationName: String,
            override val undo: CustomListAction,
        ) : Success

        @Parcelize
        data class LocationChanged(
            val customListName: CustomListName,
            override val undo: CustomListAction,
        ) : Success
    }

    @Parcelize data object GenericError : CustomListActionResultData
}
