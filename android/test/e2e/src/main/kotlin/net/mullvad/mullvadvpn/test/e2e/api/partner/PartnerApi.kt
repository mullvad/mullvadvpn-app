package net.mullvad.mullvadvpn.test.e2e.api.partner

import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.plugins.defaultRequest
import io.ktor.client.plugins.logging.LogLevel
import io.ktor.client.plugins.logging.Logging
import io.ktor.client.request.headers
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.URLProtocol.Companion.HTTPS
import io.ktor.http.contentType
import io.ktor.http.path
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import net.mullvad.mullvadvpn.test.e2e.BuildConfig
import net.mullvad.mullvadvpn.test.e2e.misc.KermitLogger

class PartnerApi(base64AuthCredentials: String) {
    private val client: HttpClient =
        HttpClient(CIO) {
            install(ContentNegotiation) { json(Json { ignoreUnknownKeys = true }) }
            install(Logging) {
                logger = KermitLogger()
                level = LogLevel.INFO
            }

            defaultRequest {
                url {
                    protocol = PROTOCOL
                    host = BASE_URL
                }
                contentType(ContentType.Application.Json)

                headers { append("Authorization", "Basic $base64AuthCredentials") }
            }
            expectSuccess = true
        }

    suspend fun createAccount(): String =
        withContext(Dispatchers.IO) {
            client.post { url { path(ACCOUNT_PATH) } }.body<CreateAccountResponse>().id
        }

    suspend fun addTime(accountNumber: String, daysToAdd: Int) =
        withContext(Dispatchers.IO) {
            val request = AddTimeRequest(daysToAdd)
            client
                .post {
                    url { path("$ACCOUNT_PATH/$accountNumber/extend") }
                    setBody(request)
                }
                .bodyAsText()
        }

    companion object {
        private val PROTOCOL = HTTPS
        private const val BASE_URL = "partner.${BuildConfig.INFRASTRUCTURE_BASE_DOMAIN}"
        private const val ACCOUNT_PATH = "${BuildConfig.API_VERSION}/accounts"
    }
}

@Serializable data class CreateAccountResponse(val id: String)

@Serializable data class AddTimeRequest(val days: Int)
