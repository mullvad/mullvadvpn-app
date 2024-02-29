package net.mullvad.mullvadvpn.compose.communication

import net.mullvad.mullvadvpn.relaylist.Location

sealed interface CustomListAction {
    data class Rename(val customListId: String, val name: String) : CustomListAction {
        fun not(oldName: String): CustomListAction = this.copy(name = oldName)
    }

    data class Delete(val customListId: String) : CustomListAction {
        fun not(name: String, locations: List<Location>): CustomListAction = Create(name, locations)
    }

    data class Create(val name: String, val locations: List<Location> = emptyList()) :
        CustomListAction {
        fun not(customListId: String) = Delete(customListId)
    }

    data class UpdateLocations(val customListId: String, val newList: Boolean) : CustomListAction {
        fun not(locations: List<Location>): CustomListAction =
            RemoveLocations(customListId, locations)
    }

    data class RemoveLocations(val customListId: String, val locations: List<Location>) :
        CustomListAction
}
