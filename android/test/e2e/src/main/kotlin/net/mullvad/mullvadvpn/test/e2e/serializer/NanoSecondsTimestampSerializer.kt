package net.mullvad.mullvadvpn.test.e2e.serializer

import kotlinx.serialization.KSerializer
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder

object NanoSecondsTimestampSerializer : KSerializer<DateTime> {
    override val descriptor: SerialDescriptor =
        PrimitiveSerialDescriptor("DateTime", PrimitiveKind.LONG)

    override fun deserialize(decoder: Decoder): DateTime {
        val long = decoder.decodeLong()
        return DateTime(long / 1000)
    }

    override fun serialize(encoder: Encoder, value: DateTime) {
        throw NotImplementedError("Only interested in deserialization")
    }
}
