package net.mullvad.mullvadvpn.test.benchmark.api.mullvad

import co.touchlab.kermit.Logger
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.plugins.defaultRequest
import io.ktor.client.plugins.logging.LogLevel
import io.ktor.client.plugins.logging.Logging
import io.ktor.client.plugins.resources.Resources
import io.ktor.client.request.bearerAuth
import io.ktor.client.request.delete
import io.ktor.client.request.get
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.HttpHeaders
import io.ktor.http.URLProtocol.Companion.HTTPS
import io.ktor.http.contentType
import io.ktor.http.path
import io.ktor.serialization.kotlinx.json.json
import io.ktor.utils.io.InternalAPI
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.withContext
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import net.mullvad.mullvadvpn.test.benchmark.BuildConfig

class MullvadApi {
    private val json = Json {
        ignoreUnknownKeys = true
        prettyPrint = true
        isLenient = true
    }
    private val client: HttpClient =
        HttpClient(CIO) {
            install(ContentNegotiation) { json(json) }
            install(Resources)
            install(Logging) {
                level = LogLevel.INFO
                sanitizeHeader { header -> header == HttpHeaders.Authorization }
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

    @OptIn(InternalAPI::class)
    suspend fun login(accountNumber: String): String =
        withContext(Dispatchers.IO) {
            val test =
                client
                    .post {
                        url { path(AUTH_PATH) }
                        setBody(LoginRequest(accountNumber))
                    }
                    .also {
                        Logger.d("Login response: ${it}")
                        Logger.d("Login response: ${it.bodyAsText()}")
                    }
                    .bodyAsText()
            json.decodeFromString<LoginResponse>(test).
                also {
                    Logger.d { "Login response: $it" }
                }.accessToken
        }

    @Serializable data class Device(val name: String, val id: String)

    suspend fun getDeviceList(accessToken: String): List<String> =
        withContext(Dispatchers.IO) {
            client
                .get {
                    url { path(DEVICES_PATH) }
                    bearerAuth(accessToken)
                }
                .body<List<Device>>()
                .map { it.id }
        }

    suspend fun removeDevice(accessToken: String, deviceId: String) =
        withContext(Dispatchers.IO) {
            client.delete {
                url { path("$DEVICES_PATH/$deviceId") }
                bearerAuth(accessToken)
            }
        }

    companion object {
        private const val BASE_URL = "api-app.${BuildConfig.INFRASTRUCTURE_BASE_DOMAIN}"
        private const val AUTH_PATH = "auth/v1/token"
        private const val DEVICES_PATH = "accounts/v1/devices"
    }
}

@OptIn(ExperimentalSerializationApi::class)
@Serializable
data class LoginRequest(@SerialName("account_number") val accountNumber: String)

@Serializable data class LoginResponse(@SerialName("access_token") val accessToken: String)

suspend fun MullvadApi.removeAllDevices(accessToken: String) =
    withContext(Dispatchers.IO) {
        val token = login(accessToken)
        val devices = getDeviceList(token)

        devices.map { async { removeDevice(token, it) } }.awaitAll()
    }
