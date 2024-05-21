package net.mullvad.mullvadvpn.compose.communication

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId

sealed interface CustomListAction : Parcelable {

    @Parcelize
    data class Rename(
        val id: net.mullvad.mullvadvpn.lib.model.CustomListId,
        val name: CustomListName,
        val newName: CustomListName
    ) : CustomListAction {
        fun not() = this.copy(name = newName, newName = name)
    }

    @Parcelize
    data class Delete(val id: net.mullvad.mullvadvpn.lib.model.CustomListId) : CustomListAction {
        fun not(
            name: CustomListName,
            locations: List<net.mullvad.mullvadvpn.lib.model.GeoLocationId>
        ) = Create(name, locations)
    }

    @Parcelize
    data class Create(
        val name: CustomListName,
        val locations: List<net.mullvad.mullvadvpn.lib.model.GeoLocationId>
    ) : CustomListAction {
        fun not(customListId: net.mullvad.mullvadvpn.lib.model.CustomListId) = Delete(customListId)
    }

    @Parcelize
    data class UpdateLocations(
        val id: net.mullvad.mullvadvpn.lib.model.CustomListId,
        val locations: List<net.mullvad.mullvadvpn.lib.model.GeoLocationId> = emptyList()
    ) : CustomListAction {
        fun not(locations: List<net.mullvad.mullvadvpn.lib.model.GeoLocationId>): UpdateLocations =
            UpdateLocations(id = id, locations = locations)
    }
}
