package net.mullvad.mullvadvpn.compose.communication

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName

@Parcelize sealed interface CustomListActionResult : Parcelable

@Parcelize data object GenericError : CustomListActionResult, Parcelable

sealed interface CustomListSuccess : CustomListActionResult, Parcelable {
    val undo: CustomListAction
}

@Parcelize
data class Created(
    val id: CustomListId,
    val name: CustomListName,
    val locationNames: List<String>,
    override val undo: CustomListAction.Delete
) : CustomListSuccess

@Parcelize
data class Deleted(override val undo: CustomListAction.Create) : CustomListSuccess {
    val name: CustomListName
        get() = undo.name
}

@Parcelize
data class Renamed(override val undo: CustomListAction.Rename) : CustomListSuccess {
    val name: CustomListName
        get() = undo.name
}

@Parcelize
data class LocationsChanged(
    val name: CustomListName,
    override val undo: CustomListAction.UpdateLocations
) : CustomListSuccess
