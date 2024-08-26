package net.mullvad.mullvadvpn.test.e2e.misc

import co.touchlab.kermit.Logger
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.HttpResponseValidator
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.plugins.defaultRequest
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
import kotlinx.serialization.Transient
import kotlinx.serialization.builtins.serializer
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encodeToString
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonContentPolymorphicSerializer
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.modules.SerializersModule
import kotlinx.serialization.modules.contextual
import net.mullvad.mullvadvpn.test.e2e.BuildConfig
import org.joda.time.DateTime
import org.joda.time.Interval
import org.junit.jupiter.api.fail

@JvmInline
@Serializable(with = PacketCaptureSessionAsStringSerializer::class)
value class PacketCaptureSession(val value: UUID = UUID.randomUUID())

object NanoSecondsTimestampAsDateTimeSerializer : KSerializer<DateTime> {
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
        defaultRequest { url("http://${BuildConfig.PACKET_CAPTURE_API_HOST}") }

        install(ContentNegotiation) {
            json(
                Json {
                    isLenient = true
                    prettyPrint = true

                    serializersModule = SerializersModule {
                        contextual(NanoSecondsTimestampAsDateTimeSerializer)
                    }
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
            httpClient.post("capture") {
                contentType(ContentType.Application.Json)
                setBody(Json.encodeToString(jsonObject))
            }
    }

    suspend fun sendStopCaptureRequest(session: PacketCaptureSession) {
        Logger.v("Sending stop capture request for session ${session.value}")
        httpClient.post("stop-capture/${session.value.toString()}")
    }

    suspend fun sendGetCapturedPacketsRequest(session: PacketCaptureSession): HttpResponse {
        val testDeviceIpAddress = Networking.getDeviceIpv4Address()
        return httpClient.put("parse-capture/${session.value.toString()}") {
            contentType(ContentType.Application.Json)
            accept(ContentType.Application.Json)
            setBody("[\"$testDeviceIpAddress\"]")
        }
    }

    suspend fun sendGetPcapFileRequest(session: PacketCaptureSession): HttpResponse {
        return httpClient.get("last-capture/${session.value.toString()}") {
            accept(ContentType.parse("application/json"))
        }
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
            val connectionInfoParts = connectionInfo.split(":")
            val ipAddress = connectionInfoParts.first()
            val port = connectionInfoParts.last().toInt()
            return Host(ipAddress, port)
        }
    }
}

object PacketSerializer : JsonContentPolymorphicSerializer<Packet>(Packet::class) {
    override fun selectDeserializer(element: JsonElement): KSerializer<out Packet> {
        return if (element.jsonObject["from_peer"]?.jsonPrimitive?.booleanOrNull!!) {
            TxPacket.serializer()
        } else {
            RxPacket.serializer()
        }
    }
}

@Serializable
data class Stream(
    @SerialName("peer_addr") private val sourceAddressAndPort: String,
    @SerialName("other_addr") private val destinationAddressAndPort: String,
    @SerialName("flow_id") val flowId: String?,
    @SerialName("transport_protocol") val transportProtocol: NetworkTransportProtocol,
    val packets: List<Packet>,
) {
    @Transient val sourceHost = Host.fromString(sourceAddressAndPort)
    @Transient val destinationHost = Host.fromString(destinationAddressAndPort)

    @Transient private val startDate: DateTime = packets.first().date
    @Transient private val endDate: DateTime = packets.last().date
    @Transient private val txStartDate: DateTime? = txPackets().firstOrNull()?.date
    @Transient private val txEndDate: DateTime? = txPackets().lastOrNull()?.date
    @Transient private val rxStartDate: DateTime? = rxPackets().firstOrNull()?.date
    @Transient private val rxEndDate: DateTime? = rxPackets().lastOrNull()?.date

    @Transient val interval = Interval(startDate, endDate)

    fun txPackets(): List<TxPacket> = packets.filterIsInstance<TxPacket>()

    fun rxPackets(): List<RxPacket> = packets.filterIsInstance<RxPacket>()

    fun txInterval(): Interval? =
        if (txStartDate != null && txEndDate != null) Interval(txStartDate, txEndDate) else null

    fun rxInterval(): Interval? =
        if (rxStartDate != null && rxEndDate != null) Interval(rxStartDate, rxEndDate) else null

    init {
        require(packets.isNotEmpty()) { "Stream must contain at least one packet" }
    }
}

@Serializable(with = PacketSerializer::class)
sealed interface Packet {
    @SerialName("timestamp") val date: DateTime
    val fromPeer: Boolean
}

@Serializable
data class RxPacket(
    @SerialName("timestamp") @Contextual override val date: DateTime,
    @SerialName("from_peer") override val fromPeer: Boolean,
) : Packet

@Serializable
data class TxPacket(
    @SerialName("timestamp") @Contextual override val date: DateTime,
    @SerialName("from_peer") override val fromPeer: Boolean,
) : Packet
