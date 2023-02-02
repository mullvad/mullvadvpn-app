package net.mullvad.mullvadvpn.test.mockapi

import android.util.Log
import net.mullvad.mullvadvpn.test.mockapi.constant.LOG_TAG
import okhttp3.mockwebserver.MockResponse
import okio.Buffer
import org.json.JSONException
import org.json.JSONObject

fun MockResponse.addJsonHeader(): MockResponse {
    return addHeader("Content-Type", "application/json")
}

fun Buffer.getAccountToken(): String? {
    return try {
        JSONObject(readUtf8()).getString("account_number")
    } catch (ex: JSONException) {
        Log.e(LOG_TAG, "Unable to parse account number")
        ex.printStackTrace()
        null
    }
}

fun Buffer.getPubKey(): String? {
    return try {
        JSONObject(readUtf8()).getString("pubkey")
    } catch (ex: JSONException) {
        null
    }
}
