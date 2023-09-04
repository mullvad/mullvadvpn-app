package net.mullvad.mullvadvpn.lib.http

import android.content.Context
import android.util.Log
import com.android.volley.Request
import com.android.volley.toolbox.JsonObjectRequest
import com.android.volley.toolbox.RequestFuture
import com.android.volley.toolbox.Volley
import org.json.JSONObject

/**
 * This class should primarily be used to make calls to the api during the early stages of
 * implementing a new endpoint. These calls should then be migrated to the daemon and this class
 * should not be used outside of this narrow scope.
 */
class MullvadHttpClient(context: Context) {
    private val queue = Volley.newRequestQueue(context)

    fun newPurchase(accountToken: String): String? {
        val authToken = login(accountToken)
        val response =
            sendSimpleSynchronousRequest(
                method = Request.Method.POST,
                url = NEW_PURCHASE_URL,
                token = authToken
            )
        return response?.getString("obfuscated_external_account_id")
    }

    fun acknowledgePurchase(
        accountToken: String,
        productId: String,
        purchaseToken: String
    ): Boolean {
        val authToken = login(accountToken)
        val response =
            sendSimpleSynchronousRequest(
                method = Request.Method.POST,
                url = ACKNOWLEDGE_PURCHASE_URL,
                token = authToken,
                body =
                    JSONObject().apply {
                        put("product_id", productId)
                        put("token", purchaseToken)
                    }
            )
        return response != null
    }

    private fun login(accountToken: String): String {
        val json = JSONObject().apply { put("account_number", accountToken) }
        return sendSimpleSynchronousRequest(Request.Method.POST, AUTH_URL, json)
            ?.getString("access_token")
            ?: ""
    }

    private fun sendSimpleSynchronousRequest(
        method: Int,
        url: String,
        body: JSONObject? = null,
        token: String? = null
    ): JSONObject? {
        val future = RequestFuture.newFuture<JSONObject>()
        val request =
            object : JsonObjectRequest(method, url, body, future, future) {
                override fun getHeaders(): MutableMap<String, String> {
                    val headers = HashMap<String, String>()
                    if (body != null) {
                        headers.put("Content-Type", "application/json")
                    }
                    if (token != null) {
                        headers.put("Authorization", "Bearer $token")
                    }
                    return headers
                }
            }
        queue.add(request)
        return try {
            future.get()
        } catch (e: Exception) {
            Log.e("Error", "Could not login", e)
            null
        }
    }

    companion object {
        private const val API_BASE_URL = "https://api.mullvad.net"
        private const val API_VERSION = "v1"
        private const val AUTH_URL = "$API_BASE_URL/auth/$API_VERSION/token"
        private const val NEW_PURCHASE_URL = "$API_BASE_URL/payments/google-play/new"
        private const val ACKNOWLEDGE_PURCHASE_URL =
            "$API_BASE_URL/payments/google-play/acknowledge"
    }
}
