package net.mullvad.mullvadvpn.test.e2e.misc

import androidx.test.platform.app.InstrumentationRegistry
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.test.e2e.constant.VALID_TEST_ACCOUNT_NUMBER_ARGUMENT_KEY
import net.mullvad.mullvadvpn.test.e2e.extension.getRequiredArgument
import net.mullvad.mullvadvpn.test.e2e.interactor.MullvadAccountInteractor
import org.junit.jupiter.api.extension.BeforeEachCallback
import org.junit.jupiter.api.extension.ExtensionContext

class CleanupAccountTestRule : BeforeEachCallback {

    override fun beforeEach(context: ExtensionContext) {
        Logger.d("Cleaning up account before test: ${context.requiredTestMethod.name}")
        val targetContext = InstrumentationRegistry.getInstrumentation().targetContext
        val validTestAccountNumber =
            InstrumentationRegistry.getArguments()
                .getRequiredArgument(VALID_TEST_ACCOUNT_NUMBER_ARGUMENT_KEY)
        MullvadAccountInteractor(SimpleMullvadHttpClient(targetContext), validTestAccountNumber)
            .cleanupAccount()
    }
}
