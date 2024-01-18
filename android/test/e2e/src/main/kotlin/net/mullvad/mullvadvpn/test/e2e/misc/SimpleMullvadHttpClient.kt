package net.mullvad.mullvadvpn.test.e2e.misc

import android.content.Context
import android.util.Log
import androidx.test.services.events.TestEventException
import com.android.volley.Request
import com.android.volley.toolbox.JsonArrayRequest
import com.android.volley.toolbox.JsonObjectRequest
import com.android.volley.toolbox.RequestFuture
import com.android.volley.toolbox.StringRequest
import com.android.volley.toolbox.Volley
import net.mullvad.mullvadvpn.test.e2e.constant.ACCOUNT_URL
import net.mullvad.mullvadvpn.test.e2e.constant.AUTH_URL
import net.mullvad.mullvadvpn.test.e2e.constant.CONN_CHECK_URL
import net.mullvad.mullvadvpn.test.e2e.constant.DEVICE_LIST_URL
import net.mullvad.mullvadvpn.test.e2e.constant.LOG_TAG
import net.mullvad.mullvadvpn.test.e2e.constant.PARTNER_ACCOUNT_URL
import org.json.JSONArray
import org.json.JSONObject

class SimpleMullvadHttpClient(context: Context) {

    private val queue = Volley.newRequestQueue(context)

    fun removeAllDevices(accountToken: String) {
        Log.v(LOG_TAG, "Remove all devices")
        val token = login(accountToken)
        val devices = getDeviceList(token)
        devices.forEach { removeDevice(token, it) }
        Log.v(LOG_TAG, "All devices removed")
    }

    fun login(accountToken: String): String {
        Log.v(LOG_TAG, "Attempt login with account token: $accountToken")
        val json = JSONObject().apply { put("account_number", accountToken) }
        return sendSimpleSynchronousRequest(Request.Method.POST, AUTH_URL, json)!!.let { response ->
            response.getString("access_token").also { accessToken ->
                Log.v(LOG_TAG, "Successfully logged in and received access token: $accessToken")
            }
        }
    }

    fun createAccount(): String {
        return sendSimpleSynchronousRequest(method = Request.Method.POST, url = ACCOUNT_URL)!!
            .getString("number")
    }

    fun addTimeToAccountUsingPartnerAuth(
        accountNumber: String,
        daysToAdd: Int,
        partnerAuth: String
    ) {
        sendSimpleSynchronousRequest(
            method = Request.Method.POST,
            url = "$PARTNER_ACCOUNT_URL/$accountNumber/extend",
            body = JSONObject().apply { put("days", "$daysToAdd") },
            authorizationHeader = "Basic $partnerAuth"
        )
    }

    fun getDeviceList(accessToken: String): List<String> {
        Log.v(LOG_TAG, "Get devices")

        val response =
            sendSimpleSynchronousRequestArray(
                Request.Method.GET,
                DEVICE_LIST_URL,
                token = accessToken
            )

        return response!!
            .iterator<JSONObject>()
            .asSequence()
            .toList()
            .also {
                it.map { jsonObject -> jsonObject.getString("name") }
                    .also { deviceNames -> Log.v(LOG_TAG, "Devices received: $deviceNames") }
            }
            .map { it.getString("id") }
            .toList()
    }

    fun removeDevice(token: String, deviceId: String) {
        Log.v(LOG_TAG, "Remove device: $deviceId")
        sendSimpleSynchronousRequestString(
            method = Request.Method.DELETE,
            url = "$DEVICE_LIST_URL/$deviceId",
            authorizationHeader = "Bearer $token"
        )
    }

    fun runConnectionCheck(): ConnCheckState? {
        return sendSimpleSynchronousRequestString(Request.Method.GET, CONN_CHECK_URL)
            ?.let { respose -> JSONObject(respose) }
            ?.let { json ->
                ConnCheckState(
                    isConnected = json.getBoolean("mullvad_exit_ip"),
                    ipAddress = json.getString("ip")
                )
            }
    }

    private fun sendSimpleSynchronousRequest(
        method: Int,
        url: String,
        body: JSONObject? = null,
        authorizationHeader: String? = null
    ): JSONObject? {
        val future = RequestFuture.newFuture<JSONObject>()
        val request =
            object : JsonObjectRequest(method, url, body, future, future) {
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
            future.get().also { response ->
                Log.v(LOG_TAG, "Json object request response: $response")
            }
        } catch (e: Exception) {
            Log.v(LOG_TAG, "Json object request error: ${e.message}")
            throw TestEventException(REQUEST_ERROR_MESSAGE)
        }
    }

    private fun sendSimpleSynchronousRequestString(
        method: Int,
        url: String,
        body: String? = null,
        authorizationHeader: String? = null
    ): String? {
        val future = RequestFuture.newFuture<String>()
        val request =
            object : StringRequest(method, url, future, future) {
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
            future.get().also { response -> Log.v(LOG_TAG, "String request response: $response") }
        } catch (e: Exception) {
            Log.v(LOG_TAG, "String request error: ${e.message}")
            throw TestEventException(REQUEST_ERROR_MESSAGE)
        }
    }

    private fun sendSimpleSynchronousRequestArray(
        method: Int,
        url: String,
        body: JSONArray? = null,
        token: String? = null
    ): JSONArray? {
        val future = RequestFuture.newFuture<JSONArray>()
        val request =
            object : JsonArrayRequest(method, url, null, future, future) {
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
            future.get().also { response ->
                Log.v(LOG_TAG, "Json array request response: $response")
            }
        } catch (e: Exception) {
            Log.v(LOG_TAG, "Json array request error: ${e.message}")
            throw TestEventException(REQUEST_ERROR_MESSAGE)
        }
    }

    operator fun <T> JSONArray.iterator(): Iterator<T> =
        (0 until this.length()).asSequence().map { this.get(it) as T }.iterator()

    companion object {
        private const val REQUEST_ERROR_MESSAGE =
            "Unable to verify account due to invalid account or connectivity issues."
    }
}
