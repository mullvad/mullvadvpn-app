package net.mullvad.mullvadvpn.test.e2e.misc

import androidx.test.platform.app.InstrumentationRegistry
import co.touchlab.kermit.Logger
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.test.api.mullvad.MullvadApi
import net.mullvad.mullvadvpn.test.api.mullvad.removeAllDevices
import net.mullvad.mullvadvpn.test.e2e.BuildConfig
import net.mullvad.mullvadvpn.test.e2e.constant.getValidAccountNumber
import org.junit.jupiter.api.extension.BeforeEachCallback
import org.junit.jupiter.api.extension.ExtensionContext

class CleanupAccountTestRule : BeforeEachCallback {
    private val mullvadApi = MullvadApi(BuildConfig.INFRASTRUCTURE_BASE_DOMAIN)

    override fun beforeEach(context: ExtensionContext) {
        Logger.d("Cleaning up account before test: ${context.requiredTestMethod.name}")
        val validTestAccountNumber = InstrumentationRegistry.getArguments().getValidAccountNumber()
        runBlocking { mullvadApi.removeAllDevices(validTestAccountNumber) }
    }
}
