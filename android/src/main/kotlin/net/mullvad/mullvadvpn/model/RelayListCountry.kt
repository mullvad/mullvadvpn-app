package net.mullvad.mullvadvpn.model

data class RelayListCountry(val name: String, val code: String, val cities: List<RelayListCity>) {
}
