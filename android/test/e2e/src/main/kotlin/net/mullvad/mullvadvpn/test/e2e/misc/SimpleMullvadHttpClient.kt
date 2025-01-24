package net.mullvad.mullvadvpn.test.e2e.misc

import android.content.Context
import androidx.test.services.events.TestEventException
import co.touchlab.kermit.Logger
import com.android.volley.Request
import com.android.volley.VolleyError
import com.android.volley.toolbox.JsonArrayRequest
import com.android.volley.toolbox.JsonObjectRequest
import com.android.volley.toolbox.RequestFuture
import com.android.volley.toolbox.StringRequest
import com.android.volley.toolbox.Volley
import net.mullvad.mullvadvpn.test.e2e.constant.AUTH_URL
import net.mullvad.mullvadvpn.test.e2e.constant.CONN_CHECK_URL
import net.mullvad.mullvadvpn.test.e2e.constant.DEVICE_LIST_URL
import net.mullvad.mullvadvpn.test.e2e.constant.PARTNER_ACCOUNT_URL
import org.json.JSONArray
import org.json.JSONObject
import org.junit.jupiter.api.fail

class SimpleMullvadHttpClient(context: Context) {

    private val queue = Volley.newRequestQueue(context)

    fun removeAllDevices(accountNumber: String) {
        Logger.v("Remove all devices")
        val token = login(accountNumber)
        val devices = getDeviceList(token)
        devices.forEach { removeDevice(token, it) }
        Logger.v("All devices removed")
    }

    fun login(accountNumber: String): String {
        Logger.v("Attempt login with account number: $accountNumber")
        val json = JSONObject().apply { put("account_number", accountNumber) }
        return sendSimpleSynchronousRequest(Request.Method.POST, AUTH_URL, json)!!.let { response ->
            response.getString("access_token").also { accessToken ->
                Logger.v("Successfully logged in and received access token: $accessToken")
            }
        }
    }

    fun createAccountUsingPartnerApi(partnerAuth: String): String {
        return sendSimpleSynchronousRequest(
                method = Request.Method.POST,
                url = PARTNER_ACCOUNT_URL,
                authorizationHeader = "Basic $partnerAuth",
            )!!
            .getString("id")
    }

    fun addTimeToAccountUsingPartnerAuth(
        accountNumber: String,
        daysToAdd: Int,
        partnerAuth: String,
    ) {
        sendSimpleSynchronousRequest(
            method = Request.Method.POST,
            url = "$PARTNER_ACCOUNT_URL/$accountNumber/extend",
            body = JSONObject().apply { put("days", "$daysToAdd") },
            authorizationHeader = "Basic $partnerAuth",
        )
    }

    fun getDeviceList(accessToken: String): List<String> {
        Logger.v("Get devices")

        val response =
            sendSimpleSynchronousRequestArray(
                Request.Method.GET,
                DEVICE_LIST_URL,
                token = accessToken,
            )

        return response!!
            .iterator<JSONObject>()
            .asSequence()
            .toList()
            .also {
                it.map { jsonObject -> jsonObject.getString("name") }
                    .also { deviceNames -> Logger.v("Devices received: $deviceNames") }
            }
            .map { it.getString("id") }
            .toList()
    }

    fun removeDevice(token: String, deviceId: String) {
        Logger.v("Remove device: $deviceId")
        sendSimpleSynchronousRequestString(
            method = Request.Method.DELETE,
            url = "$DEVICE_LIST_URL/$deviceId",
            authorizationHeader = "Bearer $token",
        )
    }

    fun runConnectionCheck(): ConnCheckState? {
        return sendSimpleSynchronousRequestString(Request.Method.GET, CONN_CHECK_URL)
            ?.let { respose -> JSONObject(respose) }
            ?.let { json ->
                ConnCheckState(
                    isConnected = json.getBoolean("mullvad_exit_ip"),
                    ipAddress = json.getString("ip"),
                )
            }
    }

    private fun sendSimpleSynchronousRequest(
        method: Int,
        url: String,
        body: JSONObject? = null,
        authorizationHeader: String? = null,
    ): JSONObject? {
        val future = RequestFuture.newFuture<JSONObject>()

        val request =
            object : JsonObjectRequest(method, url, body, future, onErrorResponse) {
                override fun getHeaders(): MutableMap<String, String> {
                    val headers = HashMap<String, String>()
                    if (body != null) {
                        headers.put("Content-Type", "application/json")
                    }
                    if (authorizationHeader != null) {
                        headers.put("Authorization", authorizationHeader)
                    }
                    return headers
                }
            }
        queue.add(request)
        return try {
            future.get().also { response -> Logger.v("Json object request response: $response") }
        } catch (e: Exception) {
            Logger.v("Json object request error: ${e.message}")
            throw TestEventException(REQUEST_ERROR_MESSAGE)
        }
    }

    private fun sendSimpleSynchronousRequestString(
        method: Int,
        url: String,
        body: String? = null,
        authorizationHeader: String? = null,
    ): String? {
        val future = RequestFuture.newFuture<String>()
        val request =
            object : StringRequest(method, url, future, onErrorResponse) {
                override fun getHeaders(): MutableMap<String, String> {
                    val headers = HashMap<String, String>()
                    if (body != null) {
                        headers.put("Content-Type", "application/json")
                    }
                    if (authorizationHeader != null) {
                        headers.put("Authorization", authorizationHeader)
                    }
                    return headers
                }
            }
        queue.add(request)
        return try {
            future.get().also { response -> Logger.v("String request response: $response") }
        } catch (e: Exception) {
            Logger.v("String request error: ${e.message}")
            throw TestEventException(REQUEST_ERROR_MESSAGE)
        }
    }

    private fun sendSimpleSynchronousRequestArray(
        method: Int,
        url: String,
        body: JSONArray? = null,
        token: String? = null,
    ): JSONArray? {
        val future = RequestFuture.newFuture<JSONArray>()
        val request =
            object : JsonArrayRequest(method, url, body, future, onErrorResponse) {
                override fun getHeaders(): MutableMap<String, String> {
                    val headers = HashMap<String, String>()
                    headers.put("Content-Type", "application/json")
                    if (token != null) {
                        headers.put("Authorization", "Bearer $token")
                    }
                    return headers
                }
            }
        queue.add(request)
        return try {
            future.get().also { response -> Logger.v("Json array request response: $response") }
        } catch (e: Exception) {
            Logger.v("Json array request error: ${e.message}")
            throw TestEventException(REQUEST_ERROR_MESSAGE)
        }
    }

    operator fun <T> JSONArray.iterator(): Iterator<T> =
        (0 until length())
            .asSequence()
            .map {
                @Suppress("UNCHECKED_CAST")
                get(it) as T
            }
            .iterator()

    companion object {
        private const val REQUEST_ERROR_MESSAGE =
            "Unable to verify account due to invalid account or connectivity issues."

        private val onErrorResponse = { error: VolleyError ->
            if (error.networkResponse != null) {
                if (error.networkResponse.statusCode == 429) {
                    fail("Request failed with response status code 429: Too many requests")
                }

                Logger.e(
                    "Response returned error message: ${error.message} " +
                        "status code: ${error.networkResponse.statusCode}"
                )
            } else {
                Logger.e("Response returned error: ${error.message}")
            }
        }
    }
}
