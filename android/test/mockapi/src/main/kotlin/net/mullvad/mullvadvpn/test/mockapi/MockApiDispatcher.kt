package net.mullvad.mullvadvpn.test.mockapi

import android.util.Log
import net.mullvad.mullvadvpn.test.mockapi.constant.ACCOUNT_URL_PATH
import net.mullvad.mullvadvpn.test.mockapi.constant.AUTH_TOKEN_URL_PATH
import net.mullvad.mullvadvpn.test.mockapi.constant.CREATE_ACCOUNT_URL_PATH
import net.mullvad.mullvadvpn.test.mockapi.constant.DEVICES_URL_PATH
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ACCESS_TOKEN
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_1
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_2
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_3
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_4
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_5
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_1
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_2
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_3
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_4
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_5
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
    var canAddDevices: Boolean = true

    private var cachedPubKeyFromAppUnderTest: String? = null

    override fun dispatch(request: RecordedRequest): MockResponse {
        Log.d(LOG_TAG, "Request: $request (body=${request.body.peek().readUtf8()})")
        return when (request.path) {
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
            "$DEVICES_URL_PATH/$DUMMY_ID_1",
            "$DEVICES_URL_PATH/$DUMMY_ID_2",
            "$DEVICES_URL_PATH/$DUMMY_ID_3",
            "$DEVICES_URL_PATH/$DUMMY_ID_4",
            "$DEVICES_URL_PATH/$DUMMY_ID_5" -> {
                when (request.method) {
                    "get",
                    "GET" -> handleDeviceInfoRequest()
                    "delete",
                    "DELETE" -> {
                        canAddDevices = true
                        MockResponse().setResponseCode(204)
                    }
                    else -> MockResponse().setResponseCode(404)
                }
            }
            ACCOUNT_URL_PATH -> handleAccountInfoRequest()
            CREATE_ACCOUNT_URL_PATH -> handleAccountCreationRequest()
            else -> MockResponse().setResponseCode(404)
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

    private fun handleDeviceInfoRequest(): MockResponse {
        return cachedPubKeyFromAppUnderTest?.let { cachedKey ->
            MockResponse()
                .setResponseCode(200)
                .addJsonHeader()
                .setBody(
                    deviceJson(
                            id = DUMMY_ID_1,
                            name = DUMMY_DEVICE_NAME_1,
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
                if (canAddDevices) {
                    MockResponse()
                        .setResponseCode(201)
                        .addJsonHeader()
                        .setBody(
                            deviceJson(
                                    id = DUMMY_ID_1,
                                    name = DUMMY_DEVICE_NAME_1,
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
            val body =
                JSONArray()
                    .put(
                        deviceJson(
                            id = DUMMY_ID_1,
                            name = DUMMY_DEVICE_NAME_1,
                            publicKey = cachedKey,
                            creationDate = currentUtcTimeWithOffsetZero().minusDays(1)
                        )
                    )
                    .put(
                        deviceJson(
                            id = DUMMY_ID_2,
                            name = DUMMY_DEVICE_NAME_2,
                            publicKey = cachedKey,
                            creationDate = currentUtcTimeWithOffsetZero().minusDays(2)
                        )
                    )
                    .put(
                        deviceJson(
                            id = DUMMY_ID_3,
                            name = DUMMY_DEVICE_NAME_3,
                            publicKey = cachedKey,
                            creationDate = currentUtcTimeWithOffsetZero().minusDays(3)
                        )
                    )
                    .put(
                        deviceJson(
                            id = DUMMY_ID_4,
                            name = DUMMY_DEVICE_NAME_4,
                            publicKey = cachedKey,
                            creationDate = currentUtcTimeWithOffsetZero().minusDays(4)
                        )
                    )
            if (canAddDevices.not()) {
                body.put(
                    deviceJson(
                        id = DUMMY_ID_5,
                        name = DUMMY_DEVICE_NAME_5,
                        publicKey = cachedKey,
                        creationDate = currentUtcTimeWithOffsetZero().minusDays(5)
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
