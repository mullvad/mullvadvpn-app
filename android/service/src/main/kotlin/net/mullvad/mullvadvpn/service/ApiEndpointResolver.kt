package net.mullvad.mullvadvpn.service

import co.touchlab.kermit.Logger
import kotlin.coroutines.cancellation.CancellationException
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import net.mullvad.mullvadvpn.lib.common.util.runBlockingWithTimeout
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpoint
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointConfiguration

class ApiEndpointResolver(private val dispatcher: CoroutineDispatcher = Dispatchers.IO) {
    fun resolve(apiEndpointConfiguration: ApiEndpointConfiguration): ApiEndpoint? =
        try {
            runBlockingWithTimeout(
                context = CoroutineScope(dispatcher).coroutineContext,
                timeout = MAX_RESOLVE_WAIT_TIME_MS
            ) {
                apiEndpointConfiguration.apiEndpoint()
            }
        } catch (e: CancellationException) {
            Logger.e("Could not resolve api endpoint configuration")
            null
        }

    companion object {
        const val MAX_RESOLVE_WAIT_TIME_MS = 5000L
    }
}
