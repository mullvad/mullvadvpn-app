package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import java.util.UUID
import kotlinx.parcelize.Parcelize

@JvmInline
@Parcelize
value class DeviceId(val value: UUID) : Parcelable {
    companion object {
        fun fromString(value: String): DeviceId = DeviceId(UUID.fromString(value))
    }
}
