package net.mullvad.mullvadvpn.lib.endpoint

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class ApiEndpointOverride(
    val hostname: String,
    val port: Int = CUSTOM_ENDPOINT_HTTPS_PORT,
    val disableTls: Boolean = false,
    val forceDirectConnection: Boolean = true,
) : Parcelable {
    companion object {
        const val CUSTOM_ENDPOINT_HTTPS_PORT = 443
    }
}
