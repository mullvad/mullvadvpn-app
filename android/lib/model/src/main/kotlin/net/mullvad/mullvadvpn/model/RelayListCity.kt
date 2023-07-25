package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class RelayListCity(val name: String, val code: String, val relays: ArrayList<Relay>) :
    Parcelable
