package net.mullvad.mullvadvpn.service.di

import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpoint
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpoint.Companion.CUSTOM_ENDPOINT_HTTPS_PORT
import net.mullvad.mullvadvpn.service.BuildConfig
import org.koin.dsl.bind
import org.koin.dsl.module

val apiEndpointModule = module {
    single {
        if (BuildConfig.FLAVOR_infrastructure != "prod") {
            ApiEndpoint.Custom(BuildConfig.API_ENDPOINT, CUSTOM_ENDPOINT_HTTPS_PORT)
        } else {
            ApiEndpoint.Default
        }
    } bind ApiEndpoint::class
}
