package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import org.joda.time.DateTime

@Parcelize
data class Device(val id: DeviceId, private val name: String, val created: DateTime) : Parcelable {
    fun displayName(): String =
        name.split(" ").joinToString(" ") { word ->
            word.replaceFirstChar { firstChar -> firstChar.uppercase() }
        }
}
