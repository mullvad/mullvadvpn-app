package net.mullvad.mullvadvpn.lib.endpoint

class ApiEndpointFromIntentHolder {
    var apiEndpointOverride: ApiEndpointOverride? = null
        private set

    fun setApiEndpointOverride(apiEndpointOverride: ApiEndpointOverride?) {
        this.apiEndpointOverride = apiEndpointOverride
    }
}
