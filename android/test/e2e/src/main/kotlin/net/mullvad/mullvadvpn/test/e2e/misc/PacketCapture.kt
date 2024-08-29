package net.mullvad.mullvadvpn.test.e2e.misc

import co.touchlab.kermit.Logger
import io.ktor.client.*
import io.ktor.client.call.body
import io.ktor.client.engine.cio.*
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.*
import io.ktor.client.statement.HttpResponse
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.serialization.kotlinx.json.json
import java.io.Serial
import java.util.Date
import java.util.UUID
import junit.framework.TestCase.fail
import kotlinx.serialization.Contextual
import kotlinx.serialization.KSerializer
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.json.*
import org.joda.time.DateTime

data class PacketCaptureSession(val identifier: String = UUID.randomUUID().toString())

class PacketCapture {
    private val client = PacketCaptureClient()

    suspend fun startCapture(): PacketCaptureSession {
        val session = PacketCaptureSession()
        client.sendStartCaptureRequest(session)
        return session
    }

    suspend fun stopCapture(session: PacketCaptureSession): StreamCollection {
        client.sendStopCaptureRequest(session)
        val parsedPacketsResponse = client.sendGetCapturedPacketsRequest(session)
        val capturedStreams = parsedPacketsResponse.body<List<Stream>>()
        Logger.v("Captured streams: $capturedStreams")
        val streamCollection = StreamCollection(capturedStreams)
        return streamCollection
    }
}

class PacketCaptureClient {
    private val baseUrl = "http://8.8.8.8"
    private val client =
        HttpClient(CIO) {
            install(ContentNegotiation) {
                json(
                    Json {
                        isLenient = true
                        prettyPrint = true
                    }
                )
            }
        }

    suspend fun sendStartCaptureRequest(session: PacketCaptureSession) {
        val jsonBody = buildJsonObject { put("label", session.identifier) }

        Logger.v("Sending start capture request with body: $jsonBody.toString()")

        val response =
            client.post("$baseUrl/capture") {
                contentType(ContentType.Application.Json)
                setBody(jsonBody.toString())
            }
    }

    suspend fun sendStopCaptureRequest(session: PacketCaptureSession) {
        val response = client.post("$baseUrl/stop-capture/${session.identifier}")
    }

    suspend fun sendGetCapturedPacketsRequest(session: PacketCaptureSession): HttpResponse {
        val testDeviceIpAddress = Networking.getTestDeviceIpAddress()
        return client.put("$baseUrl/parse-capture/${session.identifier}") {
            contentType(ContentType.Application.Json)
            accept(ContentType.Application.Json)
            setBody("[\"$testDeviceIpAddress\"]")
        }
    }
}

@Serializable
enum class NetworkTransportProtocol(val value: String) {
    @SerialName("tcp") TCP("tcp"),
    @SerialName("udp") UDP("udp"),
    @SerialName("icmp") ICMP("icmp")
}

object PacketSerializer : KSerializer<List<Packet>> {
    override val descriptor: SerialDescriptor = ListSerializer(Packet.serializer()).descriptor

    override fun deserialize(decoder: Decoder): List<Packet> {
        val jsonDecoder = decoder as? JsonDecoder ?: error("Can only be deserialized from JSON")
        val elements = jsonDecoder.decodeJsonElement().jsonArray

        return elements.map { element: JsonElement ->
            val jsonObject = element.jsonObject
            val fromPeer = jsonObject["from_peer"]?.jsonPrimitive?.booleanOrNull ?: error("Missing from_peer field")

            if (fromPeer) {
                jsonDecoder.json.decodeFromJsonElement(TxPacket.serializer(), element)
            } else {
                jsonDecoder.json.decodeFromJsonElement(RxPacket.serializer(), element)
            }
        }
    }

    override fun serialize(encoder: Encoder, value: List<Packet>) {
        throw NotImplementedError("Only interested in deserialization")
    }
}

@Serializable
data class Stream(
    @SerialName("peer_addr") val sourceAddress: String,
    @SerialName("other_addr") val destinationAddress: String,
    @SerialName("flow_id") val flowId: String?,
    @SerialName("transport_protocol") val transportProtocol: NetworkTransportProtocol,
    @Serializable(with = PacketSerializer::class) val packets: List<Packet>
) {
    @Contextual val startDate: DateTime = packets.first().date
    @Contextual val endDate: DateTime = packets.last().date

    @Contextual val txStartDate: DateTime? = packets.firstOrNull { it.fromPeer }?.date
    @Contextual val txEndDate: DateTime? = packets.lastOrNull { it.fromPeer }?.date

    @Contextual val rxStartDate: DateTime? = packets.firstOrNull { !it.fromPeer }?.date
    @Contextual val rxEndDate: DateTime? = packets.lastOrNull { !it.fromPeer }?.date
}

/*@Serializable
sealed class Packet(@SerialName("from_peer") val fromPeer: Boolean, val timestamp: String) {
    @Contextual val date = DateTime(timestamp.toLong())
    @Contextual val leakStatus = LeakStatus.UNKNOWN
}*/

@Serializable
sealed class Packet {
    abstract val timestamp: String
    abstract val fromPeer: Boolean
    abstract val date: DateTime
    abstract var leakStatus: LeakStatus
}

@Serializable
@SerialName("RxPacket")
data class RxPacket(
    @SerialName("timestamp") override val timestamp: String,
    @SerialName("from_peer") override val fromPeer: Boolean
) : Packet() {
    @Contextual override val date = DateTime(timestamp.toLong())
    @Contextual override var leakStatus = LeakStatus.UNKNOWN
}

@Serializable
@SerialName("TxPacket")
data class TxPacket(
    @SerialName("timestamp") override val timestamp: String,
    @SerialName("from_peer") override val fromPeer: Boolean
) : Packet() {
    @Contextual override val date = DateTime(timestamp.toLong())
    @Contextual override var leakStatus = LeakStatus.UNKNOWN
}
