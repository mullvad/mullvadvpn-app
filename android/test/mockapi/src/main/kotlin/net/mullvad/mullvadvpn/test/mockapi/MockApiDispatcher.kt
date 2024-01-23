package net.mullvad.mullvadvpn.test.mockapi

import android.util.Log
import net.mullvad.mullvadvpn.test.mockapi.constant.ACCOUNT_URL_PATH
import net.mullvad.mullvadvpn.test.mockapi.constant.AUTH_TOKEN_URL_PATH
import net.mullvad.mullvadvpn.test.mockapi.constant.CREATE_ACCOUNT_URL_PATH
import net.mullvad.mullvadvpn.test.mockapi.constant.DEVICES_URL_PATH
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ACCESS_TOKEN
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_1
import net.mullvad.mullvadvpn.test.mockapi.constant.LOG_TAG
import net.mullvad.mullvadvpn.test.mockapi.util.accessTokenJsonResponse
import net.mullvad.mullvadvpn.test.mockapi.util.accountCreationJson
import net.mullvad.mullvadvpn.test.mockapi.util.accountInfoJson
import net.mullvad.mullvadvpn.test.mockapi.util.currentUtcTimeWithOffsetZero
import net.mullvad.mullvadvpn.test.mockapi.util.deviceJson
import net.mullvad.mullvadvpn.test.mockapi.util.tooManyDevicesJsonResponse
import okhttp3.mockwebserver.Dispatcher
import okhttp3.mockwebserver.MockResponse
import okhttp3.mockwebserver.RecordedRequest
import okio.Buffer
import org.joda.time.DateTime
import org.json.JSONArray

class MockApiDispatcher : Dispatcher() {

    var expectedAccountToken: String? = null
    var accountExpiry: DateTime? = null
    var devices: MutableMap<String, String>? = null
    private val canAddDevices: Boolean
        get() = (devices?.size ?: 0) < 5

    var devicePendingToGetCreated: Pair<String, String>? = null

    private var cachedPubKeyFromAppUnderTest: String? = null

    override fun dispatch(request: RecordedRequest): MockResponse {
        Log.d(LOG_TAG, "Request: $request (body=${request.body.peek().readUtf8()})")
        return when (request.path ?: "") {
            AUTH_TOKEN_URL_PATH -> handleLoginRequest(request.body)
            DEVICES_URL_PATH -> {
                when (request.method) {
                    "get",
                    "GET" -> handleDeviceListRequest()
                    "post",
                    "POST" -> handleDeviceCreationRequest(request.body)
                    else -> MockResponse().setResponseCode(404)
                }
            }
            ACCOUNT_URL_PATH -> handleAccountInfoRequest()
            CREATE_ACCOUNT_URL_PATH -> handleAccountCreationRequest()
            else -> {
                if (request.path?.contains(DEVICES_URL_PATH) == true) {
                    val deviceId = request.path?.split("/")?.lastOrNull()
                    if (deviceId != null && devices?.contains(deviceId) == true) {
                        when (request.method) {
                            "get",
                            "GET" -> handleDeviceInfoRequest(deviceId)
                            "delete",
                            "DELETE" -> {
                                devices?.remove(deviceId)
                                MockResponse().setResponseCode(204)
                            }
                            else -> MockResponse().setResponseCode(404)
                        }
                    } else {
                        MockResponse().setResponseCode(404)
                    }
                } else {
                    MockResponse().setResponseCode(404)
                }
            }
        }.also { response ->
            Log.d(LOG_TAG, "Response: $response (body=${response.getBody()?.peek()?.readUtf8()})")
        }
    }

    private fun handleLoginRequest(requestBody: Buffer): MockResponse {
        val accountToken = requestBody.getAccountToken()

        return if (accountToken != null && accountToken == expectedAccountToken) {
            MockResponse()
                .setResponseCode(200)
                .addJsonHeader()
                .setBody(
                    accessTokenJsonResponse(
                            accessToken = DUMMY_ACCESS_TOKEN,
                            expiry = currentUtcTimeWithOffsetZero().plusDays(1)
                        )
                        .toString()
                )
        } else {
            Log.e(
                LOG_TAG,
                "Unexpected account token (expected=$expectedAccountToken was=$accountToken)"
            )
            MockResponse().setResponseCode(400)
        }
    }

    private fun handleAccountInfoRequest(): MockResponse {
        return accountExpiry?.let { expiry ->
            MockResponse()
                .setResponseCode(200)
                .addJsonHeader()
                .setBody(accountInfoJson(id = DUMMY_ID_1, expiry = expiry).toString())
        } ?: MockResponse().setResponseCode(400)
    }

    private fun handleDeviceInfoRequest(deviceId: String): MockResponse {
        return cachedPubKeyFromAppUnderTest?.let { cachedKey ->
            MockResponse()
                .setResponseCode(200)
                .addJsonHeader()
                .setBody(
                    deviceJson(
                            id = deviceId,
                            name = devices!![deviceId]!!, // Should always exist
                            publicKey = cachedKey,
                            creationDate = currentUtcTimeWithOffsetZero().minusDays(1)
                        )
                        .toString()
                )
        } ?: MockResponse().setResponseCode(400)
    }

    private fun handleDeviceCreationRequest(body: Buffer): MockResponse {
        return body
            .getPubKey()
            .also { newKey -> cachedPubKeyFromAppUnderTest = newKey }
            ?.let { newKey ->
                if (canAddDevices && devicePendingToGetCreated != null) {
                    MockResponse()
                        .setResponseCode(201)
                        .addJsonHeader()
                        .setBody(
                            deviceJson(
                                    id = devicePendingToGetCreated!!.first,
                                    name = devicePendingToGetCreated!!.second,
                                    publicKey = newKey,
                                    creationDate = currentUtcTimeWithOffsetZero().minusDays(1)
                                )
                                .toString()
                        )
                } else {
                    MockResponse()
                        .setResponseCode(400)
                        .addJsonHeader()
                        .setBody(tooManyDevicesJsonResponse().toString())
                }
            } ?: MockResponse().setResponseCode(400)
    }

    private fun handleDeviceListRequest(): MockResponse {
        return cachedPubKeyFromAppUnderTest?.let { cachedKey ->
            val body = JSONArray()
            devices?.onEachIndexed { index, entry ->
                body.put(
                    deviceJson(
                        id = entry.key,
                        name = entry.value,
                        publicKey = cachedKey,
                        creationDate = currentUtcTimeWithOffsetZero().minusDays(index + 1)
                    )
                )
            }
            MockResponse().setResponseCode(200).addJsonHeader().setBody(body.toString())
        } ?: MockResponse().setResponseCode(400)
    }

    private fun handleAccountCreationRequest(): MockResponse {
        return expectedAccountToken?.let { expectedAccountToken ->
            MockResponse()
                .setResponseCode(201)
                .addJsonHeader()
                .setBody(
                    accountCreationJson(
                            id = DUMMY_ID_1,
                            expiry = DateTime(),
                            accountToken = expectedAccountToken
                        )
                        .toString()
                )
        } ?: MockResponse().setResponseCode(400)
    }
}
