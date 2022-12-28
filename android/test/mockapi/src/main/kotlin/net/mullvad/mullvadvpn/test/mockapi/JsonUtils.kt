package net.mullvad.mullvadvpn.test.mockapi

import java.time.OffsetDateTime
import org.json.JSONArray
import org.json.JSONObject

fun accountInfoJson(
    id: String,
    expiry: OffsetDateTime
) = JSONObject().apply {
    put("id", id)
    put("expiry", expiry.toString())
    put("max_ports", 5)
    put("can_add_ports", true)
    put("max_devices", 5)
    put("can_add_devices", true)
}

fun deviceJson(
    id: String,
    name: String,
    publicKey: String,
    creationDate: OffsetDateTime
) = JSONObject().apply {
    put("id", id)
    put("name", name)
    put("pubkey", publicKey)
    put("hijack_dns", true)
    put("created", creationDate.toString())
    put("ipv4_address", "127.0.0.1/32")
    put("ipv6_address", "fc00::1/128")
    put("ports", JSONArray())
}

fun accessTokenJsonResponse(
    accessToken: String,
    expiry: OffsetDateTime
) = JSONObject().apply {
    put("access_token", accessToken)
    put("expiry", expiry.toString())
}
