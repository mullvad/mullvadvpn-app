package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.lib.model.extensions.startCase
import org.joda.time.DateTime

@Parcelize
data class Device(val id: DeviceId, private val name: String, val creationDate: DateTime) :
    Parcelable {
    fun displayName(): String = name.startCase()
}
