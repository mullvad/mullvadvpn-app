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
import java.util.UUID
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put

data class PacketCaptureSession(val identifier: String = UUID.randomUUID().toString())

class PacketCapture {
    private val client = PacketCaptureClient()

    suspend fun startCapture(): PacketCaptureSession {
        val session = PacketCaptureSession()
        client.sendStartCaptureRequest(session)
        return session
    }

    suspend fun stopCapture(session: PacketCaptureSession) {
        client.sendStopCaptureRequest(session)
        val parsedPacketsResponse = client.sendGetCapturedPacketsRequest(session)
        Logger.v("Parsed packet capture objects: ${parsedPacketsResponse.body<List<Stream>>()}")
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
        val testDeviceIpAddress = Networking.getIPAddress()
        return client.put("$baseUrl/parse-capture/${session.identifier}") {
            contentType(ContentType.Application.Json)
            accept(ContentType.Application.Json)
            setBody("[\"$testDeviceIpAddress\"]")
        }
    }
}

@Serializable
data class Stream(
    @SerialName("peer_addr") val sourceAddress: String,
    @SerialName("other_addr") val destinationAddress: String,
    @SerialName("flow_id") val flowId: String?,
    @SerialName("transport_protocol") val transport_protocol: String,
    val packets: List<Packet>
) {}

@Serializable
data class Packet(@SerialName("from_peer") val from_peer: String, val timestamp: String) {}
