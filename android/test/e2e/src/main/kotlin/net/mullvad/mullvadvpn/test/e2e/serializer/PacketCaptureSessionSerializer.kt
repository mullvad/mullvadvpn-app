package net.mullvad.mullvadvpn.test.e2e.serializer

import java.util.UUID
import kotlinx.serialization.KSerializer
import kotlinx.serialization.builtins.serializer
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import net.mullvad.mullvadvpn.test.e2e.router.packetCapture.PacketCaptureSession

object PacketCaptureSessionSerializer : KSerializer<PacketCaptureSession> {
    override val descriptor: SerialDescriptor = String.serializer().descriptor

    override fun deserialize(decoder: Decoder): PacketCaptureSession {
        val string = decoder.decodeString()
        return PacketCaptureSession(UUID.fromString(string))
    }

    override fun serialize(encoder: Encoder, value: PacketCaptureSession) {
        encoder.encodeString(value.value.toString())
    }
}
