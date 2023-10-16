package net.mullvad.mullvadvpn.service.di

import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointConfiguration
import net.mullvad.mullvadvpn.lib.endpoint.CUSTOM_ENDPOINT_HTTPS_PORT
import net.mullvad.mullvadvpn.lib.endpoint.CustomApiEndpointConfiguration
import net.mullvad.mullvadvpn.lib.endpoint.DefaultApiEndpointConfiguration
import net.mullvad.mullvadvpn.service.BuildConfig
import org.koin.dsl.bind
import org.koin.dsl.module

val apiEndpointModule = module {
    single {
        if (BuildConfig.FLAVOR_infrastructure != "prod") {
            CustomApiEndpointConfiguration(BuildConfig.API_ENDPOINT, CUSTOM_ENDPOINT_HTTPS_PORT)
        } else {
            DefaultApiEndpointConfiguration()
        }
    } bind ApiEndpointConfiguration::class
}
