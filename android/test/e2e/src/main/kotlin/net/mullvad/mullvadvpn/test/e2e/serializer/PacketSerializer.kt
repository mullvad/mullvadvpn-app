package net.mullvad.mullvadvpn.test.e2e.serializer

import kotlinx.serialization.KSerializer
import kotlinx.serialization.json.JsonContentPolymorphicSerializer
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import net.mullvad.mullvadvpn.test.e2e.model.Packet
import net.mullvad.mullvadvpn.test.e2e.model.RxPacket
import net.mullvad.mullvadvpn.test.e2e.model.TxPacket

object PacketSerializer : JsonContentPolymorphicSerializer<Packet>(Packet::class) {
    override fun selectDeserializer(element: JsonElement): KSerializer<out Packet> {
        return if (element.jsonObject["from_peer"]?.jsonPrimitive?.booleanOrNull!!) {
            TxPacket.serializer()
        } else {
            RxPacket.serializer()
        }
    }
}
