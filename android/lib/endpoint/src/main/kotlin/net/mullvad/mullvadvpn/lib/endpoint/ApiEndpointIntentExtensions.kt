package net.mullvad.mullvadvpn.lib.endpoint

import android.content.Intent
import android.os.Build

private const val OVERRIDE_API_EXTRA_NAME = "override_api"

fun Intent.putApiEndpointConfigurationExtra(apiEndpointConfiguration: ApiEndpointConfiguration) {
    putExtra(OVERRIDE_API_EXTRA_NAME, apiEndpointConfiguration)
}

fun Intent.getApiEndpointConfigurationExtras(): ApiEndpointConfiguration? {
    return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        getParcelableExtra(OVERRIDE_API_EXTRA_NAME, ApiEndpointConfiguration::class.java)
    } else {
        getParcelableExtra(OVERRIDE_API_EXTRA_NAME)
    }
}
