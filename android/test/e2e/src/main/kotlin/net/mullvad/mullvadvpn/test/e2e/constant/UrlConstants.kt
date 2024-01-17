package net.mullvad.mullvadvpn.test.e2e.constant

import net.mullvad.mullvadvpn.test.e2e.BuildConfig

// API URLs
const val API_BASE_URL = "https://api.${BuildConfig.INFRASTRUCTURE_BASE_DOMAIN}"
const val AUTH_URL = "$API_BASE_URL/auth/${BuildConfig.API_VERSION}/token"
const val ACCOUNT_URL = "$API_BASE_URL/accounts/${BuildConfig.API_VERSION}/accounts"
const val DEVICE_LIST_URL = "$API_BASE_URL/accounts/${BuildConfig.API_VERSION}/devices"

// Partner URLs
const val PARTNER_BASE_URL =
    "https://partner.${BuildConfig.INFRASTRUCTURE_BASE_DOMAIN}/${BuildConfig.API_VERSION}"
const val PARTNER_ACCOUNT_URL = "$PARTNER_BASE_URL/accounts"

// Connection check
const val CONN_CHECK_URL = "https://am.i.${BuildConfig.INFRASTRUCTURE_BASE_DOMAIN}/json"
