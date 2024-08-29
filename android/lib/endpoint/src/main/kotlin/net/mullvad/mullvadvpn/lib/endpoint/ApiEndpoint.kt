package net.mullvad.mullvadvpn.lib.endpoint

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed interface ApiEndpoint : Parcelable {

    @Parcelize
    data class Custom(
        val hostname: String,
        val port: Int,
        val disableAddressCache: Boolean = true,
        val disableTls: Boolean = false
    ) : ApiEndpoint

    @Parcelize data object Default : ApiEndpoint

    // Used by jni
    fun isDefault() = this is Default

    companion object {
        const val CUSTOM_ENDPOINT_HTTPS_PORT = 443
    }
}
