package net.mullvad.mullvadvpn.test.e2e.serializer

import java.time.Instant
import java.time.ZoneId
import java.time.ZonedDateTime
import kotlinx.serialization.KSerializer
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder

object NanoSecondsTimestampSerializer : KSerializer<ZonedDateTime> {
    override val descriptor: SerialDescriptor =
        PrimitiveSerialDescriptor("ZonedDateTime", PrimitiveKind.LONG)

    override fun deserialize(decoder: Decoder): ZonedDateTime {
        val long = decoder.decodeLong()
        return ZonedDateTime.ofInstant(Instant.ofEpochMilli(long / 1000), ZoneId.systemDefault())
    }

    override fun serialize(encoder: Encoder, value: ZonedDateTime) {
        throw NotImplementedError("Only interested in deserialization")
    }
}
