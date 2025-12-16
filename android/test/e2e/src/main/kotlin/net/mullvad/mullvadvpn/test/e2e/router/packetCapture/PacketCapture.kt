package net.mullvad.mullvadvpn.test.e2e.router.packetCapture

import androidx.test.platform.app.InstrumentationRegistry
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
import kotlinx.serialization.json.Json
import net.mullvad.mullvadvpn.test.e2e.constant.getRaasHost
import net.mullvad.mullvadvpn.test.e2e.misc.Networking
import net.mullvad.mullvadvpn.test.e2e.serializer.PacketCaptureSessionSerializer
import org.junit.jupiter.api.fail

@JvmInline
@Serializable(with = PacketCaptureSessionSerializer::class)
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
        return parsedPacketsResponse.body<List<Stream>>().also { Logger.v("Captured streams: $it") }
    }

    private suspend fun getPcap(): ByteArray {
        return client.sendGetPcapFileRequest(session).body<ByteArray>()
    }

    suspend fun capturePackets(block: suspend () -> Unit): PacketCaptureResult {
        startCapture()
        block()
        stopCapture()
        return PacketCaptureResult(getParsedCapture(), getPcap())
    }
}

private fun defaultHttpClient(): HttpClient =
    HttpClient(CIO) {
        defaultRequest { url("http://${InstrumentationRegistry.getArguments().getRaasHost()}") }
        engine { requestTimeout = REQUEST_TIMEOUT_MS }

        install(ContentNegotiation) {
            json(
                Json {
                    ignoreUnknownKeys = true
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

        httpClient.post("capture") {
            contentType(ContentType.Application.Json)
            setBody(Json.encodeToString(jsonObject))
        }
    }

    suspend fun sendStopCaptureRequest(session: PacketCaptureSession) {
        Logger.v("Sending stop capture request for session ${session.value}")
        httpClient.post("stop-capture/${session.value}")
    }

    suspend fun sendGetCapturedPacketsRequest(session: PacketCaptureSession): HttpResponse {
        val testDeviceIpAddress = Networking.getDeviceIpAddrs().ipv4
        return httpClient.put("parse-capture/${session.value}") {
            contentType(ContentType.Application.Json)
            accept(ContentType.Application.Json)
            setBody("[\"$testDeviceIpAddress\"]")
        }
    }

    suspend fun sendGetPcapFileRequest(session: PacketCaptureSession): HttpResponse {
        return httpClient.get("last-capture/${session.value}") {
            accept(ContentType.parse("application/json"))
        }
    }
}

data class PacketCaptureResult(val streams: List<Stream>, val pcap: ByteArray)

@Serializable data class StartCaptureRequestJson(val label: PacketCaptureSession)

// 30 seconds timeout, double the default timeout
private const val REQUEST_TIMEOUT_MS = 30000L
