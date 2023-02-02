package net.mullvad.mullvadvpn.test.mockapi

import android.util.Log
import net.mullvad.mullvadvpn.test.mockapi.constant.ACCOUNT_URL_PATH
import net.mullvad.mullvadvpn.test.mockapi.constant.AUTH_TOKEN_URL_PATH
import net.mullvad.mullvadvpn.test.mockapi.constant.DEVICES_URL_PATH
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ACCESS_TOKEN
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID
import net.mullvad.mullvadvpn.test.mockapi.constant.LOG_TAG
import net.mullvad.mullvadvpn.test.mockapi.util.currentUtcTimeWithOffsetZero
import okhttp3.mockwebserver.Dispatcher
import okhttp3.mockwebserver.MockResponse
import okhttp3.mockwebserver.RecordedRequest
import okio.Buffer
import org.joda.time.DateTime
import org.json.JSONArray

class MockApiDispatcher : Dispatcher() {

    var expectedAccountToken: String? = null
    var accountExpiry: DateTime? = null

    private var cachedPubKeyFromAppUnderTest: String? = null

    override fun dispatch(request: RecordedRequest): MockResponse {
        Log.d(LOG_TAG, "Request: $request (body=${request.body.peek().readUtf8()})")
        return when (request.path) {
            AUTH_TOKEN_URL_PATH -> handleLoginRequest(request.body)
            DEVICES_URL_PATH -> {
                when (request.method) {
                    "get", "GET" -> handleDeviceListRequest()
                    "post", "POST" -> handleDeviceCreationRequest(request.body)
                    else -> MockResponse().setResponseCode(404)
                }
            }
            "$DEVICES_URL_PATH/$DUMMY_ID" -> handleDeviceInfoRequest()
            ACCOUNT_URL_PATH -> handleAccountInfoRequest()
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
                    ).toString()
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
                .setBody(
                    accountInfoJson(
                        id = DUMMY_ID,
                        expiry = expiry
                    ).toString()
                )
        } ?: MockResponse().setResponseCode(400)
    }

    private fun handleDeviceInfoRequest(): MockResponse {
        return cachedPubKeyFromAppUnderTest?.let { cachedKey ->
            MockResponse()
                .setResponseCode(200)
                .addJsonHeader()
                .setBody(
                    deviceJson(
                        id = DUMMY_ID,
                        name = DUMMY_DEVICE_NAME,
                        publicKey = cachedKey,
                        creationDate = currentUtcTimeWithOffsetZero().minusDays(1)
                    ).toString()
                )
        } ?: MockResponse().setResponseCode(400)
    }

    private fun handleDeviceCreationRequest(body: Buffer): MockResponse {
        return body.getPubKey()
            .also { newKey ->
                cachedPubKeyFromAppUnderTest = newKey
            }
            ?.let { newKey ->
                MockResponse()
                    .setResponseCode(201)
                    .addJsonHeader()
                    .setBody(
                        deviceJson(
                            id = DUMMY_ID,
                            name = DUMMY_DEVICE_NAME,
                            publicKey = newKey,
                            creationDate = currentUtcTimeWithOffsetZero().minusDays(1)
                        ).toString()
                    )
            } ?: MockResponse().setResponseCode(400)
    }

    private fun handleDeviceListRequest(): MockResponse {
        return cachedPubKeyFromAppUnderTest?.let { cachedKey ->
            MockResponse()
                .setResponseCode(200)
                .addJsonHeader()
                .setBody(
                    JSONArray().put(
                        deviceJson(
                            id = DUMMY_ID,
                            name = DUMMY_DEVICE_NAME,
                            publicKey = cachedKey,
                            creationDate = currentUtcTimeWithOffsetZero().minusDays(1)
                        )
                    ).toString()
                )
        } ?: MockResponse().setResponseCode(400)
    }
}
