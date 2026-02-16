package net.mullvad.mullvadvpn.feature.anticensorship.impl.selectport

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.lib.model.PortType

@Parcelize data class SelectPortNavArgs(val portType: PortType) : Parcelable
