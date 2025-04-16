package net.mullvad.mullvadvpn.test.e2e.router.firewall

import co.touchlab.kermit.Logger
import io.ktor.client.HttpClient
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.plugins.defaultRequest
import io.ktor.client.request.delete
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.http.ContentType
import io.ktor.http.URLProtocol
import io.ktor.http.contentType
import io.ktor.serialization.kotlinx.json.json
import kotlinx.serialization.json.Json
import kotlinx.serialization.modules.SerializersModule
import kotlinx.serialization.modules.contextual
import net.mullvad.mullvadvpn.test.e2e.BuildConfig
import net.mullvad.mullvadvpn.test.e2e.serializer.NanoSecondsTimestampSerializer

class FirewallClient(private val httpClient: HttpClient = defaultHttpClient()) {
    suspend fun createRule(rule: DropRule) {
        Logger.v(
            "Sending create rule request with body: ${Json.encodeToString(DropRule.serializer(), rule)}"
        )
        Logger.v(
            "Requesting firewall API to block ${rule.protocols} traffic from ${rule.source} to ${rule.destination}"
        )
        httpClient.post("rule") {
            contentType(ContentType.Application.Json)
            setBody(Json.encodeToString(DropRule.serializer(), rule))
        }
    }

    suspend fun removeAllRules() {
        Logger.v("Sending remove all rules request")
        httpClient.delete("remove-rules/${SessionIdentifier.fromDeviceIdentifier()}")
    }
}

private fun defaultHttpClient(): HttpClient =
    HttpClient(CIO) {
        defaultRequest {
            url {
                protocol = URLProtocol.HTTP
                host = BuildConfig.TEST_ROUTER_API_HOST
            }
        }

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
        expectSuccess = true
    }
