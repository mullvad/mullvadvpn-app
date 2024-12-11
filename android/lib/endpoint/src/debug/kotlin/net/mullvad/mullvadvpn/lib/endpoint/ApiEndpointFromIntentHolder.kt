package net.mullvad.mullvadvpn.lib.endpoint

import android.content.Intent

class ApiEndpointFromIntentHolder {
    var apiEndpointOverride: ApiEndpointOverride? = null
        private set

    fun handleIntent(intent: Intent) {
        apiEndpointOverride = intent.getApiEndpointConfigurationExtras()
    }
}
