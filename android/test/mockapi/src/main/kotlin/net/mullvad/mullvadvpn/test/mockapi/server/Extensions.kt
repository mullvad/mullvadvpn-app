package net.mullvad.mullvadvpn.test.mockapi.server

import co.touchlab.kermit.Logger
import io.ktor.http.ContentType
import io.ktor.http.HttpStatusCode
import io.ktor.server.response.respond
import io.ktor.server.response.respondText
import io.ktor.server.routing.RoutingCall
import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject

fun String.getAccountNumber(): String? {
    return try {
        JSONObject(this).getString("account_number")
    } catch (ex: JSONException) {
        Logger.e("Unable to parse account number", ex)
        null
    }
}

fun String.getPubKey(): String? {
    return try {
        JSONObject(this).getString("pubkey")
    } catch (ex: JSONException) {
        Logger.e("Unable to parse pubkey", ex)
        null
    }
}

suspend fun RoutingCall.respondOkJson(jsonObject: JSONObject) =
    respondText(
        text = jsonObject.toString(),
        contentType = ContentType.Application.Json,
        status = HttpStatusCode.OK,
    )

suspend fun RoutingCall.respondCreatedJson(jsonObject: JSONObject) =
    respondText(
        text = jsonObject.toString(),
        contentType = ContentType.Application.Json,
        status = HttpStatusCode.OK,
    )

suspend fun RoutingCall.respondOkJson(jsonArray: JSONArray) =
    respondText(
        text = jsonArray.toString(),
        contentType = ContentType.Application.Json,
        status = HttpStatusCode.OK,
    )

suspend fun RoutingCall.respondErrorJson(jsonObject: JSONObject) =
    respondText(
        text = jsonObject.toString(),
        contentType = ContentType.Application.Json,
        status = HttpStatusCode.BadRequest,
    )

suspend fun RoutingCall.respondError() = respond(HttpStatusCode.BadRequest)
