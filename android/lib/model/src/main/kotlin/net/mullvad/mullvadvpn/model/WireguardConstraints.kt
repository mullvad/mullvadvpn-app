package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize data class WireguardConstraints(val port: Constraint<Port>) : Parcelable
