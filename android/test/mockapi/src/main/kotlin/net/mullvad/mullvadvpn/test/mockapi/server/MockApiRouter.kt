package net.mullvad.mullvadvpn.test.mockapi.server

import androidx.test.platform.app.InstrumentationRegistry
import co.touchlab.kermit.Logger
import io.ktor.http.ContentType
import io.ktor.http.HttpStatusCode
import io.ktor.server.application.Application
import io.ktor.server.request.receiveText
import io.ktor.server.response.respond
import io.ktor.server.response.respondText
import io.ktor.server.routing.RoutingCall
import io.ktor.server.routing.delete
import io.ktor.server.routing.get
import io.ktor.server.routing.post
import io.ktor.server.routing.routing
import java.time.ZonedDateTime
import net.mullvad.mullvadvpn.test.mockapi.R
import net.mullvad.mullvadvpn.test.mockapi.constant.ACCOUNT_URL_PATH
import net.mullvad.mullvadvpn.test.mockapi.constant.AUTH_TOKEN_URL_PATH
import net.mullvad.mullvadvpn.test.mockapi.constant.CREATE_ACCOUNT_URL_PATH
import net.mullvad.mullvadvpn.test.mockapi.constant.DEVICES_ID_URL_PATH
import net.mullvad.mullvadvpn.test.mockapi.constant.DEVICES_URL_PATH
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ACCESS_TOKEN
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_1
import net.mullvad.mullvadvpn.test.mockapi.constant.RELAY_LIST_URL_PATH
import net.mullvad.mullvadvpn.test.mockapi.util.accessTokenJsonResponse
import net.mullvad.mullvadvpn.test.mockapi.util.accountCreationJson
import net.mullvad.mullvadvpn.test.mockapi.util.accountInfoJson
import net.mullvad.mullvadvpn.test.mockapi.util.deviceJson
import net.mullvad.mullvadvpn.test.mockapi.util.tooManyDevicesJsonResponse
import org.json.JSONArray

class MockApiRouter {

    var expectedAccountNumber: String? = null
    var accountExpiry: ZonedDateTime? = null
    var devices: MutableMap<String, String>? = null
    private val canAddDevices: Boolean
        get() = (devices?.size ?: 0) < 5

    var devicePendingToGetCreated: Pair<String, String>? = null

    private var cachedPubKeyFromAppUnderTest: String? = null

    private val relayListJson =
        InstrumentationRegistry.getInstrumentation()
            .context
            .resources
            .openRawResource(R.raw.relay_list)
            .bufferedReader()
            .readText()

    fun setup(application: Application) {
        application.routing {
            post(AUTH_TOKEN_URL_PATH) { handleLoginRequest(call) }
            get(DEVICES_URL_PATH) { handleDeviceListRequest(call) }
            post(DEVICES_URL_PATH) { handleDeviceCreationRequest(call) }
            get(ACCOUNT_URL_PATH) { handleAccountInfoRequest(call) }
            post(CREATE_ACCOUNT_URL_PATH) { handleAccountCreationRequest(call) }
            get(RELAY_LIST_URL_PATH) { handleGetRelayListRequest(call) }
            get(DEVICES_ID_URL_PATH) { handleDeviceInfoRequest(call) }
            delete(DEVICES_ID_URL_PATH) { handleDeviceDeletionRequest(call) }
        }
    }

    private suspend fun handleLoginRequest(call: RoutingCall) {
        val requestBody = call.receiveText()
        val accountNumber = requestBody.getAccountNumber()

        return if (accountNumber != null && accountNumber == expectedAccountNumber) {
            call.respondOkJson(
                accessTokenJsonResponse(
                    accessToken = DUMMY_ACCESS_TOKEN,
                    expiry = ZonedDateTime.now().plusHours(24),
                )
            )
        } else {
            Logger.e(
                "Unexpected account number (expected=$expectedAccountNumber was=$accountNumber)"
            )
            call.respondError()
        }
    }

    private suspend fun handleAccountInfoRequest(call: RoutingCall) {
        return accountExpiry?.let { expiry ->
            call.respondOkJson(accountInfoJson(id = DUMMY_ID_1, expiry = expiry))
        } ?: call.respondError()
    }

    private suspend fun handleDeviceInfoRequest(call: RoutingCall) {
        cachedPubKeyFromAppUnderTest?.let { cachedKey ->
            val deviceId = call.parameters["deviceId"]!!
            call.respondOkJson(
                deviceJson(
                    id = deviceId,
                    name = devices!![deviceId]!!, // Should always exist
                    publicKey = cachedKey,
                    creationDate = ZonedDateTime.now().minusDays(1),
                )
            )
        } ?: call.respondError()
    }

    private suspend fun handleDeviceCreationRequest(call: RoutingCall) {
        val requestBody = call.receiveText()
        requestBody
            .getPubKey()
            .also { newKey -> cachedPubKeyFromAppUnderTest = newKey }
            ?.let { newKey ->
                if (canAddDevices && devicePendingToGetCreated != null) {
                    call.respondCreatedJson(
                        deviceJson(
                            id = devicePendingToGetCreated!!.first,
                            name = devicePendingToGetCreated!!.second,
                            publicKey = newKey,
                            creationDate = ZonedDateTime.now().minusDays(1),
                        )
                    )
                } else {
                    call.respondErrorJson(tooManyDevicesJsonResponse())
                }
            } ?: call.respondError()
    }

    private suspend fun handleDeviceListRequest(call: RoutingCall) {
        return cachedPubKeyFromAppUnderTest?.let { cachedKey ->
            val body = JSONArray()
            devices?.onEachIndexed { index, entry ->
                body.put(
                    deviceJson(
                        id = entry.key,
                        name = entry.value,
                        publicKey = cachedKey,
                        creationDate = ZonedDateTime.now().minusDays((index + 1).toLong()),
                    )
                )
            }
            call.respondOkJson(body)
        } ?: call.respondError()
    }

    private suspend fun handleAccountCreationRequest(call: RoutingCall) {
        return expectedAccountNumber?.let { expectedAccountNumber ->
            call.respondCreatedJson(
                accountCreationJson(
                    id = DUMMY_ID_1,
                    expiry = ZonedDateTime.now(),
                    accountNumber = expectedAccountNumber,
                )
            )
        } ?: call.respondError()
    }

    private suspend fun handleGetRelayListRequest(call: RoutingCall) {
        call.respondText(
            text = relayListJson,
            contentType = ContentType.Application.Json,
            status = HttpStatusCode.OK,
        )
    }

    private suspend fun handleDeviceDeletionRequest(call: RoutingCall) {
        val deviceId = call.parameters["deviceId"]!!
        devices?.let { devices ->
            devices.remove(deviceId)
            call.respond(HttpStatusCode.NoContent)
        } ?: call.respondError()
    }
}
