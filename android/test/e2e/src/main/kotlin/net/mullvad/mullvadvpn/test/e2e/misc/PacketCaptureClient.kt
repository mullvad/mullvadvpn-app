package net.mullvad.mullvadvpn.test.e2e.misc

import io.ktor.client.*
import io.ktor.client.engine.cio.*
import io.ktor.client.request.*
import io.ktor.client.statement.HttpResponse
import io.ktor.http.ContentType
import io.ktor.http.contentType
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import java.util.UUID
import co.touchlab.kermit.Logger
import io.ktor.client.statement.bodyAsText
import net.mullvad.mullvadvpn.compose.state.DeviceListUiState

data class PacketCaptureSession(val identifier: String = UUID.randomUUID().toString())

class PacketCaptureClient {
    val baseUrl = "http://8.8.8.8"
    val client = HttpClient(CIO)

    suspend fun sendStartCaptureRequest(session: PacketCaptureSession) {
        val jsonBody = buildJsonObject {
            put("label", session.identifier)
        }

        Logger.v("Sending start capture request with body: $jsonBody.toString()")

        val response = client.post("$baseUrl/capture") {
            contentType(ContentType.Application.Json)
            setBody(jsonBody.toString())
        }
    }

    suspend fun sendStopCaptureRequest(session: PacketCaptureSession) {
        val response = client.post("$baseUrl/stop-capture/${session.identifier}")
    }

    suspend fun sendGetCapturedPacketsRequest(session: PacketCaptureSession): HttpResponse {
        val testDeviceIpAddress = Networking.getIPAddress()
        return client.put("$baseUrl/parse-capture/${session.identifier}") {
            contentType(ContentType.Application.Json)
            setBody("[\"$testDeviceIpAddress\"]")
        }
    }
}
