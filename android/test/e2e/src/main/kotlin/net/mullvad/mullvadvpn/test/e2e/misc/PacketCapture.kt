package net.mullvad.mullvadvpn.test.e2e.misc

import co.touchlab.kermit.Logger
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.HttpResponseValidator
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.accept
import io.ktor.client.request.get
import io.ktor.client.request.post
import io.ktor.client.request.put
import io.ktor.client.request.setBody
import io.ktor.client.statement.HttpResponse
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.serialization.kotlinx.json.json
import java.util.UUID
import kotlinx.serialization.Contextual
import kotlinx.serialization.KSerializer
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.builtins.serializer
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encodeToString
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonDecoder
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import org.joda.time.DateTime
import org.junit.jupiter.api.fail

@JvmInline
@Serializable(with = PacketCaptureSessionAsStringSerializer::class)
value class PacketCaptureSession(val value: UUID = UUID.randomUUID())

object PacketCaptureSessionAsStringSerializer : KSerializer<PacketCaptureSession> {
    override val descriptor: SerialDescriptor = String.serializer().descriptor

    override fun deserialize(decoder: Decoder): PacketCaptureSession {
        val string = decoder.decodeString()
        return PacketCaptureSession(UUID.fromString(string))
    }

    override fun serialize(encoder: Encoder, value: PacketCaptureSession) {
        encoder.encodeString(value.value.toString())
    }
}

class PacketCapture {
    private val client = PacketCaptureClient()
    private val session = PacketCaptureSession()

    private suspend fun startCapture() {
        client.sendStartCaptureRequest(session)
    }

    private suspend fun stopCapture() {
        client.sendStopCaptureRequest(session)
    }

    private suspend fun getParsedCapture(): List<Stream> {
        val parsedPacketsResponse = client.sendGetCapturedPacketsRequest(session)
        val capturedStreams = parsedPacketsResponse.body<List<Stream>>()
        Logger.v("Captured streams: $capturedStreams")
        return capturedStreams
    }

    private suspend fun getPcap(): ByteArray {
        return client.sendGetPcapFileRequest(session).body<ByteArray>()
    }

    suspend fun capturePackets(block: suspend () -> Unit): PacketCaptureResult {
        startCapture()
        block()
        stopCapture()
        val parsedCapture = getParsedCapture()
        val pcap = getPcap()
        return PacketCaptureResult(parsedCapture, pcap)
    }
}

private fun defaultHttpClient(): HttpClient =
    HttpClient(CIO) {
        install(ContentNegotiation) {
            json(
                Json {
                    isLenient = true
                    prettyPrint = true
                }
            )
        }

        HttpResponseValidator {
            validateResponse { response ->
                val statusCode = response.status.value
                if (statusCode >= 400) {
                    fail(
                        "Request failed with response status code $statusCode: ${response.body<String>()}"
                    )
                }
            }
            handleResponseExceptionWithRequest { exception, _ ->
                fail("Request failed to be sent with exception: ${exception.message}")
            }
        }
    }

class PacketCaptureClient(private val httpClient: HttpClient = defaultHttpClient()) {
    suspend fun sendStartCaptureRequest(session: PacketCaptureSession) {
        val jsonObject = StartCaptureRequestJson(session)

        Logger.v("Sending start capture request with body: ${Json.encodeToString(jsonObject)}")

        val response =
            httpClient.post("$BASE_URL/capture") {
                contentType(ContentType.Application.Json)
                setBody(Json.encodeToString(jsonObject))
            }
    }

    suspend fun sendStopCaptureRequest(session: PacketCaptureSession) {
        Logger.v("Sending stop capture request for session ${session.value}")
        httpClient.post("$BASE_URL/stop-capture/${session.value.toString()}")
    }

    suspend fun sendGetCapturedPacketsRequest(session: PacketCaptureSession): HttpResponse {
        val testDeviceIpAddress = Networking.getIpAddress()
        return httpClient.put("$BASE_URL/parse-capture/${session.value.toString()}") {
            contentType(ContentType.Application.Json)
            accept(ContentType.Application.Json)
            setBody("[\"$testDeviceIpAddress\"]")
        }
    }

    suspend fun sendGetPcapFileRequest(session: PacketCaptureSession): HttpResponse {
        return httpClient.get("$BASE_URL/last-capture/${session.value.toString()}") {
            // contentType(ContentType.parse("application/pcap"))
            accept(ContentType.parse("application/json"))
        }
    }

    companion object {
        const val BASE_URL = "http://192.168.105.1"
    }
}

data class PacketCaptureResult(val streams: List<Stream>, val pcap: ByteArray)

@Serializable data class StartCaptureRequestJson(val label: PacketCaptureSession)

@Serializable
enum class NetworkTransportProtocol {
    @SerialName("tcp") TCP,
    @SerialName("udp") UDP,
    @SerialName("icmp") ICMP,
}

data class Host(val ipAddress: String, val port: Int) {
    companion object {
        fun fromString(connectionInfo: String): Host {
            val ipAddress = connectionInfo.split(":").first()
            val port = connectionInfo.split(":").last().toInt()
            return Host(ipAddress, port)
        }
    }
}

object PacketSerializer : KSerializer<List<Packet>> {
    override val descriptor: SerialDescriptor = ListSerializer(Packet.serializer()).descriptor

    override fun deserialize(decoder: Decoder): List<Packet> {
        val jsonDecoder = decoder as? JsonDecoder ?: error("Can only be deserialized from JSON")
        val elements = jsonDecoder.decodeJsonElement().jsonArray

        return elements.map { element: JsonElement ->
            val jsonObject = element.jsonObject
            val fromPeer =
                jsonObject["from_peer"]?.jsonPrimitive?.booleanOrNull
                    ?: error("Missing from_peer field")

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
    @SerialName("peer_addr") private val sourceAddressAndPort: String,
    @SerialName("other_addr") private val destinationAddressAndPort: String,
    @SerialName("flow_id") val flowId: String?,
    @SerialName("transport_protocol") val transportProtocol: NetworkTransportProtocol,
    @Serializable(with = PacketSerializer::class) val packets: List<Packet>,
) {
    @Contextual public val sourceHost = Host.fromString(sourceAddressAndPort)
    @Contextual public val destinationHost = Host.fromString(destinationAddressAndPort)

    @Contextual val startDate: DateTime = packets.first().date
    @Contextual val endDate: DateTime = packets.last().date
    @Contextual val txStartDate: DateTime? = packets.firstOrNull { it.fromPeer }?.date
    @Contextual val txEndDate: DateTime? = packets.lastOrNull { it.fromPeer }?.date
    @Contextual val rxStartDate: DateTime? = packets.firstOrNull { !it.fromPeer }?.date
    @Contextual val rxEndDate: DateTime? = packets.lastOrNull { !it.fromPeer }?.date

    fun txPackets(): List<TxPacket> = packets.filterIsInstance<TxPacket>()

    fun rxPackets(): List<RxPacket> = packets.filterIsInstance<RxPacket>()
}

@Serializable
sealed interface Packet {
    val timestamp: String
    val fromPeer: Boolean
    val date: DateTime
}

@Serializable
@SerialName("RxPacket")
data class RxPacket(
    @SerialName("timestamp") override val timestamp: String,
    @SerialName("from_peer") override val fromPeer: Boolean,
) : Packet {
    @Contextual override val date = DateTime(timestamp.toLong() / 1000)
}

@Serializable
@SerialName("TxPacket")
data class TxPacket(
    @SerialName("timestamp") override val timestamp: String,
    @SerialName("from_peer") override val fromPeer: Boolean,
) : Packet {
    @Contextual override val date = DateTime(timestamp.toLong() / 1000)
}
