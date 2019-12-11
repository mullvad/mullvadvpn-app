package net.mullvad.mullvadvpn.dataproxy

import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.async
import net.mullvad.mullvadvpn.service.MullvadDaemon

class WwwAuthTokenRetriever(val daemon: Deferred<MullvadDaemon>) {
    suspend fun getAuthToken() = GlobalScope.async(Dispatchers.Default) {
        // returning an empty string is valid in case of any failures
        daemon.await().getWwwAuthToken()
    }
}
