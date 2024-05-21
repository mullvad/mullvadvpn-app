package net.mullvad.mullvadvpn.lib.model

import arrow.optics.optics

@optics
data class CustomList(
    val id: net.mullvad.mullvadvpn.lib.model.CustomListId,
    val name: CustomListName,
    val locations: List<net.mullvad.mullvadvpn.lib.model.GeoLocationId>
) {
    companion object
}
