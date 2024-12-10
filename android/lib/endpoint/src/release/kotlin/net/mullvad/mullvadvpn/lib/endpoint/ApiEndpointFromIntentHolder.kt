package net.mullvad.mullvadvpn.lib.endpoint

import android.content.Intent

// Overridding the API endpoint is not supported in release builds
class ApiEndpointFromIntentHolder {
    val apiEndpointOverride: ApiEndpointOverride? = null

    fun handleIntent(intent: Intent) {
        // No-op
    }
}
