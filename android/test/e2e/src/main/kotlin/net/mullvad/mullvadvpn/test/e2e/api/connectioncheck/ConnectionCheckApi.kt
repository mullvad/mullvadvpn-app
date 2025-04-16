package net.mullvad.mullvadvpn.test.e2e.api.connectioncheck

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
import net.mullvad.mullvadvpn.test.e2e.BuildConfig
import net.mullvad.mullvadvpn.test.e2e.misc.KermitLogger

class ConnectionCheckApi {
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

    suspend fun connectionCheck(): ConnCheckResult =
        withContext(Dispatchers.IO) {
            client.get { url { path(JSON_PATH) } }.body<ConnCheckResult>()
        }

    companion object {
        // Connection check
        private const val BASE_URL = "am.i.${BuildConfig.INFRASTRUCTURE_BASE_DOMAIN}"
        private const val JSON_PATH = "json"
    }
}
