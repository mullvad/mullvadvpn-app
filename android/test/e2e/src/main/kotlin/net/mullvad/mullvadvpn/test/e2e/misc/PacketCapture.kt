package net.mullvad.mullvadvpn.test.e2e.misc

import co.touchlab.kermit.Logger
import io.ktor.client.*
import io.ktor.client.call.body
import io.ktor.client.engine.cio.*
import io.ktor.client.plugins.HttpResponseValidator
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.*
import io.ktor.client.statement.HttpResponse
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.serialization.kotlinx.json.json
import java.util.UUID
import kotlinx.coroutines.runBlocking
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
import org.junit.jupiter.api.fail

data class PacketCaptureSession(val identifier: String = UUID.randomUUID().toString())

class PacketCapture {
    private val client = PacketCaptureClient()
    private var sessionUUID = UUID.randomUUID()

    private suspend fun startCapture() {
        client.sendStartCaptureRequest(sessionUUID)
    }

    private suspend fun stopCapture() {
        client.sendStopCaptureRequest(sessionUUID)
    }

    private suspend fun getParsedCapture(): List<Stream> {
        val parsedPacketsResponse = client.sendGetCapturedPacketsRequest(sessionUUID)
        val capturedStreams = parsedPacketsResponse.body<List<Stream>>()
        Logger.v("Captured streams: $capturedStreams")
        return capturedStreams
    }

    private suspend fun getPcap(): String {
        val pcapContent = client.sendGetPcapFileRequest(sessionUUID)
        Logger.v("PCAP content: ${pcapContent.body<String>()}")
        return pcapContent.body<String>()
    }

    suspend fun capturePackets(block: suspend () -> Unit): PacketCaptureResult = runBlocking {
        startCapture()
        block()
        stopCapture()
        val parsedCapture = getParsedCapture()
        val pcapString = getPcap()
        return@runBlocking PacketCaptureResult(parsedCapture, pcapString.toByteArray())
    }
}

class PacketCaptureClient {
    private val baseUrl = "http://192.168.105.1"
    private val httpClient =
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

    suspend fun sendStartCaptureRequest(sessionUUID: UUID) {
        val jsonBody = buildJsonObject { put("label", sessionUUID.toString()) }

        Logger.v("Sending start capture request with body: $jsonBody.toString()")

        val response =
            httpClient.post("$baseUrl/capture") {
                contentType(ContentType.Application.Json)
                setBody(jsonBody.toString())
            }
    }

    suspend fun sendStopCaptureRequest(sessionUUID: UUID) {
        val response = httpClient.post("$baseUrl/stop-capture/${sessionUUID.toString()}")
    }

    suspend fun sendGetCapturedPacketsRequest(sessionUUID: UUID): HttpResponse {
        val testDeviceIpAddress = Networking.getIpAddress()
        return httpClient.put("$baseUrl/parse-capture/${sessionUUID.toString()}") {
            contentType(ContentType.Application.Json)
            accept(ContentType.Application.Json)
            setBody("[\"$testDeviceIpAddress\"]")
        }
    }

    suspend fun sendGetPcapFileRequest(sessionUUID: UUID): HttpResponse {
        return httpClient.get("$baseUrl/last-capture/${sessionUUID.toString()}") {
            // contentType(ContentType.parse("application/pcap"))
            accept(ContentType.parse("application/json"))
        }
    }
}

data class PacketCaptureResult(val streams: List<Stream>, val pcap: ByteArray)

@Serializable
enum class NetworkTransportProtocol() {
    @SerialName("tcp") TCP,
    @SerialName("udp") UDP,
    @SerialName("icmp") ICMP,
}

data class Host(val ipAddress: String, val port: Int)

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
    @Contextual
    public val sourceHost: Host =
        Host(
            sourceAddressAndPort.split(":").first(),
            sourceAddressAndPort.split(":").last().toInt(),
        )
    @Contextual
    public val destinationHost: Host =
        Host(
            destinationAddressAndPort.split(":").first(),
            destinationAddressAndPort.split(":").last().toInt(),
        )

    @Contextual val startDate: DateTime = packets.first().date
    @Contextual val endDate: DateTime = packets.last().date
    @Contextual val txStartDate: DateTime? = packets.firstOrNull { it.fromPeer }?.date
    @Contextual val txEndDate: DateTime? = packets.lastOrNull { it.fromPeer }?.date
    @Contextual val rxStartDate: DateTime? = packets.firstOrNull { !it.fromPeer }?.date
    @Contextual val rxEndDate: DateTime? = packets.lastOrNull { !it.fromPeer }?.date

    fun getTxPackets(): List<TxPacket> = packets.filterIsInstance<TxPacket>()

    fun getRxPackets(): List<RxPacket> = packets.filterIsInstance<RxPacket>()
}

@Serializable
sealed class Packet {
    abstract val timestamp: String
    abstract val fromPeer: Boolean
    abstract val date: DateTime
}

@Serializable
@SerialName("RxPacket")
data class RxPacket(
    @SerialName("timestamp") override val timestamp: String,
    @SerialName("from_peer") override val fromPeer: Boolean,
) : Packet() {
    @Contextual override val date = DateTime(timestamp.toLong() / 1000)
}

@Serializable
@SerialName("TxPacket")
data class TxPacket(
    @SerialName("timestamp") override val timestamp: String,
    @SerialName("from_peer") override val fromPeer: Boolean,
) : Packet() {
    @Contextual override val date = DateTime(timestamp.toLong() / 1000)
}
