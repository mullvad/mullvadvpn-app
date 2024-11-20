package net.mullvad.mullvadvpn.test.e2e.misc

import android.annotation.SuppressLint
import android.provider.Settings
import androidx.test.platform.app.InstrumentationRegistry
import co.touchlab.kermit.Logger
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.HttpResponseValidator
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.plugins.defaultRequest
import io.ktor.client.request.delete
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.serialization.kotlinx.json.json
import java.util.UUID
import kotlinx.serialization.EncodeDefault
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.modules.SerializersModule
import kotlinx.serialization.modules.contextual
import net.mullvad.mullvadvpn.test.e2e.BuildConfig
import net.mullvad.mullvadvpn.test.e2e.serializer.NanoSecondsTimestampSerializer
import org.junit.jupiter.api.fail

class FirewallClient(private val httpClient: HttpClient = defaultHttpClient()) {
    suspend fun createRule(rule: FirewallRule) {
        Logger.v(
            "Sending create rule request with body: ${Json.encodeToString(FirewallRule.serializer(), rule)}"
        )
        Logger.v(
            "Requesting firewall API to block ${rule.protocols} traffic from ${rule.from} to ${rule.to}"
        )
        httpClient.post("rule") {
            contentType(ContentType.Application.Json)
            setBody(Json.encodeToString(FirewallRule.serializer(), rule))
        }
    }

    suspend fun removeAllRules() {
        Logger.v("Sending remove all rules request")
        httpClient.delete("remove-rules/${sessionIdentifierFromDeviceIdentifier()}")
    }

    @SuppressLint("HardwareIds")
    @OptIn(ExperimentalSerializationApi::class)
    @Serializable
    data class FirewallRule
    constructor(
        val from: String,
        val to: String,
        val src: String,
        val dst: String,
        val protocols: List<NetworkingProtocol>,
        @EncodeDefault
        val label: String = "urn:uuid:" + FirewallClient.sessionIdentifierFromDeviceIdentifier(),
    ) {
        companion object {
            fun blockUDPTrafficRule(to: String): FirewallRule {
                val testDeviceIpAddress = Networking.getDeviceIpv4Address()
                return FirewallRule(
                    testDeviceIpAddress,
                    to,
                    testDeviceIpAddress,
                    to,
                    listOf(NetworkingProtocol.UDP),
                )
            }
        }
    }

    @Serializable
    enum class NetworkingProtocol {
        @SerialName("tcp") TCP,
        @SerialName("udp") UDP,
        @SerialName("icmp") ICMP,
    }

    companion object {
        @SuppressLint("HardwareIds")
        fun sessionIdentifierFromDeviceIdentifier(): String {
            val deviceIdentifier =
                Settings.Secure.getString(
                    InstrumentationRegistry.getInstrumentation().targetContext.contentResolver,
                    Settings.Secure.ANDROID_ID,
                )

            return UUID.nameUUIDFromBytes(deviceIdentifier.toByteArray()).toString()
        }
    }
}

private fun defaultHttpClient(): HttpClient =
    HttpClient(CIO) {
        defaultRequest { url("http://${BuildConfig.FIREWALL_API_HOST}") }

        install(ContentNegotiation) {
            json(
                Json {
                    isLenient = true
                    prettyPrint = true

                    serializersModule = SerializersModule {
                        contextual(NanoSecondsTimestampSerializer)
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
