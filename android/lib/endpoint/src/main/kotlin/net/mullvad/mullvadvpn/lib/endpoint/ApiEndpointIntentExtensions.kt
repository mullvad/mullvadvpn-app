package net.mullvad.mullvadvpn.lib.endpoint

import android.content.Intent
import android.os.Build

private const val OVERRIDE_API_EXTRA_NAME = "override_api"

fun Intent.putApiEndpointConfigurationExtra(apiEndpointConfiguration: ApiEndpoint.Custom) {
    putExtra(OVERRIDE_API_EXTRA_NAME, apiEndpointConfiguration)
}

fun Intent.getApiEndpointConfigurationExtras(): ApiEndpoint.Custom? {
    return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        getParcelableExtra(OVERRIDE_API_EXTRA_NAME, ApiEndpoint.Custom::class.java)
    } else {
        getParcelableExtra(OVERRIDE_API_EXTRA_NAME)
    }
}
