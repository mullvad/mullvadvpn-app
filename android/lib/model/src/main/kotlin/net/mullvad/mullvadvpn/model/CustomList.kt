package net.mullvad.mullvadvpn.model

import arrow.optics.optics

@optics
data class CustomList(
    val id: CustomListId,
    val name: CustomListName,
    val locations: List<GeoLocationId>
) {
    companion object
}
