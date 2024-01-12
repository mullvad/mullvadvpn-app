package net.mullvad.mullvadvpn.test.mockapi.util

import org.joda.time.DateTime
import org.json.JSONObject

fun accountInfoJson(id: String, expiry: DateTime) =
    JSONObject().apply {
        put("id", id)
        put("expiry", expiry.formatStrictlyAccordingToIso8601AndRfc3339())
        put("max_devices", 5)
        put("can_add_devices", true)
    }

fun accountCreationJson(id: String, accountToken: String, expiry: DateTime) =
    accountInfoJson(id, expiry).apply { put("number", accountToken) }

fun deviceJson(id: String, name: String, publicKey: String, creationDate: DateTime) =
    JSONObject().apply {
        put("id", id)
        put("name", name)
        put("pubkey", publicKey)
        put("hijack_dns", true)
        put("created", creationDate.formatStrictlyAccordingToIso8601AndRfc3339())
        put("ipv4_address", "127.0.0.1/32")
        put("ipv6_address", "fc00::1/128")
    }

fun accessTokenJsonResponse(accessToken: String, expiry: DateTime) =
    JSONObject().apply {
        put("access_token", accessToken)
        put("expiry", expiry.formatStrictlyAccordingToIso8601AndRfc3339())
    }
