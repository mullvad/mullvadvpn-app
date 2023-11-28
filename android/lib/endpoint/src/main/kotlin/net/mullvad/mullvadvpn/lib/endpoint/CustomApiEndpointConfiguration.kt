package net.mullvad.mullvadvpn.lib.endpoint

import java.net.InetSocketAddress
import kotlinx.parcelize.Parcelize

const val CUSTOM_ENDPOINT_HTTPS_PORT = 443

@Parcelize
data class CustomApiEndpointConfiguration(
    val hostname: String,
    val port: Int,
    val disableAddressCache: Boolean = true,
    val disableTls: Boolean = false,
    val forceDirectConnection: Boolean = true
) : ApiEndpointConfiguration {
    override fun apiEndpoint() =
        ApiEndpoint(
            address = InetSocketAddress(hostname, port),
            disableAddressCache = disableAddressCache,
            disableTls = disableTls,
            forceDirectConnection = forceDirectConnection
        )
}
