package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import java.util.UUID
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.KSerializer
import kotlinx.serialization.Serializable
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder

@JvmInline
@Parcelize
value class DeviceId(val value: UUID) : Parcelable {
    companion object {
        fun fromString(value: String): DeviceId = DeviceId(UUID.fromString(value))
    }
}
