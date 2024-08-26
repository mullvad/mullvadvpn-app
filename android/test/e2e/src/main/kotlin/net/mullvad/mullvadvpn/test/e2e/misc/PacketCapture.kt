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
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.modules.SerializersModule
import kotlinx.serialization.modules.contextual
import net.mullvad.mullvadvpn.test.e2e.BuildConfig
import net.mullvad.mullvadvpn.test.e2e.model.Stream
import net.mullvad.mullvadvpn.test.e2e.serializer.NanoSecondsTimestampAsDateTimeSerializer
import net.mullvad.mullvadvpn.test.e2e.serializer.PacketCaptureSessionAsStringSerializer
import org.junit.jupiter.api.fail

@JvmInline
@Serializable(with = PacketCaptureSessionAsStringSerializer::class)
value class PacketCaptureSession(val value: UUID = UUID.randomUUID())

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
