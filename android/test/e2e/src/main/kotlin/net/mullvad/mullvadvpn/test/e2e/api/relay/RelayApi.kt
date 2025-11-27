package net.mullvad.mullvadvpn.test.e2e.api.relay

import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.plugins.defaultRequest
import io.ktor.client.plugins.logging.LogLevel
import io.ktor.client.plugins.logging.Logging
import io.ktor.client.request.get
import io.ktor.http.ContentType
import io.ktor.http.URLProtocol.Companion.HTTPS
import io.ktor.http.contentType
import io.ktor.http.path
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import net.mullvad.mullvadvpn.test.common.misc.RelayProvider
import net.mullvad.mullvadvpn.test.e2e.BuildConfig
import net.mullvad.mullvadvpn.test.e2e.misc.KermitLogger

class RelayApi {
    private val relayProvider = RelayProvider(BuildConfig.FLAVOR_billing)

    private val client: HttpClient =
        HttpClient(CIO) {
            install(ContentNegotiation) { json(Json { ignoreUnknownKeys = true }) }
            install(Logging) {
                logger = KermitLogger()
                level = LogLevel.INFO
            }
            defaultRequest {
                url {
                    protocol = HTTPS
                    host = BASE_URL
                }
                contentType(ContentType.Application.Json)
            }
            expectSuccess = true
        }

    suspend fun getDefaultRelayIpAddress(): String =
        withContext(Dispatchers.IO) {
            val body = client.get { url { path(RELAY_PATH) } }.body<String>()
            val ipRegex =
                """${relayProvider.getDefaultRelay().relay}.+?ipv4_addr_in":"(.+?)""""
                    .toRegex(RegexOption.DOT_MATCHES_ALL)

            ipRegex.find(body)?.groups?.get(1)?.value
                ?: error(
                    "Could not find ${relayProvider.getDefaultRelay().relay} IP address in relay list"
                )
        }

    companion object {
        private const val BASE_URL = "api-app.${BuildConfig.INFRASTRUCTURE_BASE_DOMAIN}"
        private const val RELAY_PATH = "app/v1/relays"
    }
}
