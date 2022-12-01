package net.mullvad.mullvadvpn.lib.endpoint

import kotlinx.parcelize.Parcelize

@Parcelize
data class CustomApiEndpointConfiguration(
    val apiEndpoint: ApiEndpoint
) : ApiEndpointConfiguration {
    override fun apiEndpoint() = apiEndpoint
}
