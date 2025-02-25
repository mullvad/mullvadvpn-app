package net.mullvad.mullvadvpn.test.mockapi.util

import java.time.ZonedDateTime
import org.json.JSONObject

fun accountInfoJson(id: String, expiry: ZonedDateTime) =
    JSONObject().apply {
        put("id", id)
        put("expiry", expiry.formatStrictlyAccordingToIso8601AndRfc3339())
        put("max_devices", 5)
        put("can_add_devices", true)
    }

fun accountCreationJson(id: String, accountNumber: String, expiry: ZonedDateTime) =
    accountInfoJson(id, expiry).apply { put("number", accountNumber) }

fun deviceJson(id: String, name: String, publicKey: String, creationDate: ZonedDateTime) =
    JSONObject().apply {
        put("id", id)
        put("name", name)
        put("pubkey", publicKey)
        put("hijack_dns", true)
        put("created", creationDate.formatStrictlyAccordingToIso8601AndRfc3339())
        put("ipv4_address", "127.0.0.1/32")
        put("ipv6_address", "fc00::1/128")
    }

fun accessTokenJsonResponse(accessToken: String, expiry: ZonedDateTime) =
    JSONObject().apply {
        put("access_token", accessToken)
        put("expiry", expiry.formatStrictlyAccordingToIso8601AndRfc3339())
    }

fun tooManyDevicesJsonResponse() =
    JSONObject().apply {
        put("code", "MAX_DEVICES_REACHED")
        put("detail", "This account already has the maximum number of devices.")
    }
