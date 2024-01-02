package net.mullvad.mullvadvpn.test.e2e.misc

import android.util.Log
import androidx.test.platform.app.InstrumentationRegistry
import net.mullvad.mullvadvpn.test.e2e.constant.LOG_TAG
import net.mullvad.mullvadvpn.test.e2e.constant.VALID_TEST_ACCOUNT_TOKEN_ARGUMENT_KEY
import net.mullvad.mullvadvpn.test.e2e.extension.getRequiredArgument
import net.mullvad.mullvadvpn.test.e2e.interactor.MullvadAccountInteractor
import org.junit.jupiter.api.extension.BeforeEachCallback
import org.junit.jupiter.api.extension.ExtensionContext

class CleanupAccountTestRule : BeforeEachCallback {

    override fun beforeEach(context: ExtensionContext) {
        Log.d(LOG_TAG, "Cleaning up account before test: ${context.requiredTestMethod.name}")
        val targetContext = InstrumentationRegistry.getInstrumentation().targetContext
        val validTestAccountToken =
            InstrumentationRegistry.getArguments()
                .getRequiredArgument(VALID_TEST_ACCOUNT_TOKEN_ARGUMENT_KEY)
        MullvadAccountInteractor(SimpleMullvadHttpClient(targetContext), validTestAccountToken)
            .cleanupAccount()
    }
}
