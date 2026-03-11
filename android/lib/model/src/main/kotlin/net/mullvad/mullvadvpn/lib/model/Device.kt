package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import java.time.ZonedDateTime
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.KSerializer
import kotlinx.serialization.Serializable
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import net.mullvad.mullvadvpn.lib.model.extensions.startCase

@Parcelize
@Serializable
data class Device(
    val id: DeviceId,
    private val name: String,
    val creationDate: @Serializable(with = ZonedDateTimeSerializer::class) ZonedDateTime,
) : Parcelable {
    fun displayName(): String = name.startCase()
}

object ZonedDateTimeSerializer : KSerializer<ZonedDateTime> {
    // Define how it looks in JSON (as a plain String)
    override val descriptor: SerialDescriptor =
        PrimitiveSerialDescriptor("java.time.ZonedDateTime", PrimitiveKind.STRING)

    // How to convert UUID to a String
    override fun serialize(encoder: Encoder, value: ZonedDateTime) {
        encoder.encodeString(value.toString())
    }

    // How to parse a String back into a UUID
    override fun deserialize(decoder: Decoder): ZonedDateTime {
        return ZonedDateTime.parse(decoder.decodeString())
    }
}
