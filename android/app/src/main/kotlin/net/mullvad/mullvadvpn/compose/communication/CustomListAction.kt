package net.mullvad.mullvadvpn.compose.communication

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed interface CustomListAction : Parcelable {

    @Parcelize
    data class Rename(val customListId: String, val name: String) : CustomListAction {
        fun not(): CustomListAction = this
    }

    @Parcelize
    data class Delete(val customListId: String, val name: String) : CustomListAction {
        fun not(locations: List<String>): CustomListAction = Create(name, locations)
    }

    @Parcelize
    data class Create(
        val name: String = "",
        val locations: List<String> = emptyList(),
        val locationNames: List<String> = emptyList()
    ) : CustomListAction, Parcelable {
        fun not(customListId: String) = Delete(customListId, name)
    }

    @Parcelize
    data class UpdateLocations(
        val customListId: String,
        val newList: Boolean = false,
        val locations: List<String> = emptyList()
    ) : CustomListAction {
        fun not(locations: List<String>): CustomListAction =
            UpdateLocations(customListId = customListId, locations = locations)
    }
}
