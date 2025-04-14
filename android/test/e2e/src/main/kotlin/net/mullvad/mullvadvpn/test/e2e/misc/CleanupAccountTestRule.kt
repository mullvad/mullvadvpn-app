package net.mullvad.mullvadvpn.test.e2e.misc

import androidx.test.platform.app.InstrumentationRegistry
import co.touchlab.kermit.Logger
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.test.e2e.api.mullvad.MullvadApi
import net.mullvad.mullvadvpn.test.e2e.api.mullvad.removeAllDevices
import net.mullvad.mullvadvpn.test.e2e.constant.VALID_TEST_ACCOUNT_NUMBER_ARGUMENT_KEY
import net.mullvad.mullvadvpn.test.e2e.extension.getRequiredArgument
import org.junit.jupiter.api.extension.BeforeEachCallback
import org.junit.jupiter.api.extension.ExtensionContext

class CleanupAccountTestRule : BeforeEachCallback {
    private val mullvadApi = MullvadApi()

    override fun beforeEach(context: ExtensionContext) {
        Logger.d("Cleaning up account before test: ${context.requiredTestMethod.name}")
        val validTestAccountNumber =
            InstrumentationRegistry.getArguments()
                .getRequiredArgument(VALID_TEST_ACCOUNT_NUMBER_ARGUMENT_KEY)
        runBlocking { mullvadApi.removeAllDevices(validTestAccountNumber) }
    }
}
