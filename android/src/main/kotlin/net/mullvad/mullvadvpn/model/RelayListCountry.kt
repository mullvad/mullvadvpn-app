package net.mullvad.mullvadvpn.model

import java.util.ArrayList

data class RelayListCountry(
    val name: String,
    val code: String,
    val cities: ArrayList<RelayListCity>
)
