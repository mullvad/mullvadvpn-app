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
@Serializable
value class DeviceId(@Serializable(with = UUIDSerializer::class) val value: UUID) : Parcelable {
    companion object {
        fun fromString(value: String): DeviceId = DeviceId(UUID.fromString(value))
    }
}

object UUIDSerializer : KSerializer<UUID> {
    // Define how it looks in JSON (as a plain String)
    override val descriptor: SerialDescriptor =
        PrimitiveSerialDescriptor("java.util.UUID", PrimitiveKind.STRING)

    // How to convert UUID to a String
    override fun serialize(encoder: Encoder, value: UUID) {
        encoder.encodeString(value.toString())
    }

    // How to parse a String back into a UUID
    override fun deserialize(decoder: Decoder): UUID {
        return UUID.fromString(decoder.decodeString())
    }
}
