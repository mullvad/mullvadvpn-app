package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import java.util.UUID

@JvmInline
@Parcelize
value class DeviceId(val value: UUID): Parcelable {
    companion object {
        fun fromString(value: String): DeviceId = DeviceId(UUID.fromString(value))
    }
}