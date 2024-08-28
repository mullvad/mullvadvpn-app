package net.mullvad.mullvadvpn.compose.communication

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId

sealed interface CustomListAction : Parcelable {
    @Parcelize
    data class Rename(val id: CustomListId, val name: CustomListName, val newName: CustomListName) :
        CustomListAction {
        fun not() = this.copy(name = newName, newName = name)
    }

    @Parcelize
    data class Delete(val id: CustomListId) : CustomListAction {
        fun not(name: CustomListName, locations: List<GeoLocationId>) = Create(name, locations)
    }

    @Parcelize
    data class Create(val name: CustomListName, val locations: List<GeoLocationId>) :
        CustomListAction {
        fun not(customListId: CustomListId) = Delete(customListId)
    }

    @Parcelize
    data class UpdateLocations(
        val id: CustomListId,
        val locations: List<GeoLocationId> = emptyList(),
    ) : CustomListAction {
        fun not(locations: List<GeoLocationId>): UpdateLocations =
            UpdateLocations(id = id, locations = locations)
    }
}
