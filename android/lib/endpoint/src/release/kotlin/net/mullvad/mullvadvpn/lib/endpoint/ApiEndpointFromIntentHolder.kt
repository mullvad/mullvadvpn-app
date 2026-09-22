package net.mullvad.mullvadvpn.lib.endpoint

// Overriding the API endpoint is not supported in release builds
class ApiEndpointFromIntentHolder {
    val apiEndpointOverride: ApiEndpointOverride? = null

    @Suppress("UnusedParameter")
    fun setApiEndpointOverride(apiEndpointOverride: ApiEndpointOverride?) {
        // No-op
    }
}
