package net.mullvad.mullvadvpn.lib.endpoint

import android.content.Intent
import android.os.Build

private const val OVERRIDE_API_EXTRA_NAME = "override_api"

fun Intent.putApiEndpointConfigurationExtra(apiEndpointOverrideConfiguration: ApiEndpointOverride) {
    putExtra(OVERRIDE_API_EXTRA_NAME, apiEndpointOverrideConfiguration)
}

fun Intent.getApiEndpointConfigurationExtras(): ApiEndpointOverride? {
    return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        getParcelableExtra(OVERRIDE_API_EXTRA_NAME, ApiEndpointOverride::class.java)
    } else {
        @Suppress("DEPRECATION") getParcelableExtra(OVERRIDE_API_EXTRA_NAME)
    }
}
