package net.mullvad.mullvadvpn.lib.endpoint

import kotlinx.parcelize.Parcelize

@Parcelize
class DefaultApiEndpointConfiguration : ApiEndpointConfiguration {
    override fun apiEndpoint(): ApiEndpoint? = null
}
