package net.mullvad.mullvadvpn.test.e2e.misc

import android.util.Log
import androidx.test.platform.app.InstrumentationRegistry
import net.mullvad.mullvadvpn.test.e2e.constant.LOG_TAG
import net.mullvad.mullvadvpn.test.e2e.constant.VALID_TEST_ACCOUNT_TOKEN_ARGUMENT_KEY
import net.mullvad.mullvadvpn.test.e2e.extension.getRequiredArgument
import net.mullvad.mullvadvpn.test.e2e.interactor.MullvadAccountInteractor
import org.junit.rules.TestWatcher
import org.junit.runner.Description

class CleanupAccountTestRule : TestWatcher() {
    override fun starting(description: Description) {
        Log.d(LOG_TAG, "Cleaning up account before test: ${description.methodName}")
        val targetContext = InstrumentationRegistry.getInstrumentation().targetContext
        val validTestAccountToken = InstrumentationRegistry.getArguments()
            .getRequiredArgument(VALID_TEST_ACCOUNT_TOKEN_ARGUMENT_KEY)
        MullvadAccountInteractor(SimpleMullvadHttpClient(targetContext), validTestAccountToken)
            .cleanupAccount()
    }
}
