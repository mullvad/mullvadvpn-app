package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import java.time.ZonedDateTime
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.lib.model.extensions.startCase

@Parcelize
data class Device(val id: DeviceId, private val name: String, val creationDate: ZonedDateTime) :
    Parcelable {
    fun displayName(): String = name.startCase()
}
