package net.mullvad.mullvadvpn.e2e.interactor

import net.mullvad.mullvadvpn.e2e.misc.SimpleMullvadHttpClient

class MullvadAccountInteractor(
    private val httpClient: SimpleMullvadHttpClient,
    private val testAccountToken: String
) {
    fun cleanupAccount() {
        httpClient.removeAllDevices(testAccountToken)
    }
}
