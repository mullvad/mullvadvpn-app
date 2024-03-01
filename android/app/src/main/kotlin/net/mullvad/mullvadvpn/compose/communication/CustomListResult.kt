package net.mullvad.mullvadvpn.compose.communication

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed interface CustomListResult : Parcelable {
    val undo: CustomListAction

    @Parcelize
    data class Created(
        val id: String,
        val name: String,
        val locationName: String?,
        override val undo: CustomListAction.Delete
    ) : CustomListResult

    @Parcelize
    data class Deleted(override val undo: CustomListAction.Create) : CustomListResult {
        val name
            get() = undo.name
    }

    @Parcelize
    data class Renamed(override val undo: CustomListAction.Rename) : CustomListResult {
        val name: String
            get() = undo.name
    }

    @Parcelize
    data class LocationsChanged(
        val name: String,
        override val undo: CustomListAction.UpdateLocations
    ) : CustomListResult
}
