package net.mullvad.mullvadvpn.test.e2e.router.firewall

import androidx.test.platform.app.InstrumentationRegistry
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
import net.mullvad.mullvadvpn.test.e2e.constant.getRaasHost

class FirewallClient(private val httpClient: HttpClient = defaultHttpClient()) {
    suspend fun createRule(rule: DropRule) {
        Logger.v(
            "Sending create rule request with body: ${Json.encodeToString(DropRule.serializer(), rule)}"
        )
        Logger.v(
            "Requesting firewall API to block ${rule.protocols} traffic from ${rule.source} to ${rule.destination}"
        )
        val response =
            httpClient.post("rule") {
                contentType(ContentType.Application.Json)
                setBody(Json.encodeToString(DropRule.serializer(), rule))
            }
        Logger.v("Create rule response: ${response.status.value}")
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
                host = InstrumentationRegistry.getArguments().getRaasHost()
            }
        }

        install(ContentNegotiation) {
            json(
                Json {
                    ignoreUnknownKeys = true
                    isLenient = true
                    prettyPrint = true
                }
            )
        }
        expectSuccess = true
    }
