package net.mullvad.mullvadvpn.compose.communication

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed interface CustomListAction : Parcelable {

    @Parcelize
    data class Rename(val customListId: String, val name: String, val newName: String) :
        CustomListAction {
        fun not() = this.copy(name = newName, newName = name)
    }

    @Parcelize
    data class Delete(val customListId: String) : CustomListAction {
        fun not(name: String, locations: List<String>) = Create(name, locations)
    }

    @Parcelize
    data class Create(val name: String = "", val locations: List<String> = emptyList()) :
        CustomListAction, Parcelable {
        fun not(customListId: String) = Delete(customListId)
    }

    @Parcelize
    data class UpdateLocations(
        val customListId: String,
        val locations: List<String> = emptyList()
    ) : CustomListAction {
        fun not(locations: List<String>): UpdateLocations =
            UpdateLocations(customListId = customListId, locations = locations)
    }
}
