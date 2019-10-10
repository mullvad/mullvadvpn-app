package net.mullvad.mullvadvpn.dataproxy

import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.MullvadDaemon

class WwwAuthTokenRetriever(val asyncDaemon: Deferred<MullvadDaemon>) {
    private var daemon: MullvadDaemon? = null
    private val setUpJob = setUp()

    private fun setUp() = GlobalScope.launch(Dispatchers.Default) {
        daemon = asyncDaemon.await()
    }

    suspend fun getAuthToken(): String {
        setUpJob.join()
        // returning an empty string is valid in case of any failures
        return daemon?.getWwwAuthToken() ?: ""
    }
}
