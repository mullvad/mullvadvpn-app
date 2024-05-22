package net.mullvad.mullvadvpn.compose.communication

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName

sealed interface CustomListResult : Parcelable {
    val undo: CustomListAction

    @Parcelize
    data class Created(
        val id: net.mullvad.mullvadvpn.lib.model.CustomListId,
        val name: CustomListName,
        val locationNames: List<String>,
        override val undo: CustomListAction.Delete
    ) : CustomListResult

    @Parcelize
    data class Deleted(override val undo: CustomListAction.Create) : CustomListResult {
        val name: CustomListName
            get() = undo.name
    }

    @Parcelize
    data class Renamed(override val undo: CustomListAction.Rename) : CustomListResult {
        val name: CustomListName
            get() = undo.name
    }

    @Parcelize
    data class LocationsChanged(
        val name: CustomListName,
        override val undo: CustomListAction.UpdateLocations
    ) : CustomListResult
}
