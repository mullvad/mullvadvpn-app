package net.mullvad.mullvadvpn.model

import java.util.ArrayList

data class RelayListCity(val name: String, val code: String, val relays: ArrayList<Relay>)
