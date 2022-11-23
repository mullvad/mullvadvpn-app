package net.mullvad.mullvadvpn.test.e2e

import android.content.Context
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.runner.AndroidJUnit4
import androidx.test.uiautomator.UiDevice
import net.mullvad.mullvadvpn.test.e2e.constant.INVALID_TEST_ACCOUNT_TOKEN_ARGUMENT_KEY
import net.mullvad.mullvadvpn.test.e2e.constant.VALID_TEST_ACCOUNT_TOKEN_ARGUMENT_KEY
import net.mullvad.mullvadvpn.test.e2e.extension.getRequiredArgument
import net.mullvad.mullvadvpn.test.e2e.interactor.AppInteractor
import net.mullvad.mullvadvpn.test.e2e.interactor.WebViewInteractor
import net.mullvad.mullvadvpn.test.e2e.misc.CaptureScreenshotOnFailedTestRule
import org.junit.Before
import org.junit.Rule
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
abstract class EndToEndTest {

    @Rule
    @JvmField
    val rule = CaptureScreenshotOnFailedTestRule()

    lateinit var device: UiDevice
    lateinit var targetContext: Context
    lateinit var app: AppInteractor
    lateinit var web: WebViewInteractor
    lateinit var validTestAccountToken: String
    lateinit var invalidTestAccountToken: String

    @Before
    fun setup() {
        device = UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())
        targetContext = InstrumentationRegistry.getInstrumentation().targetContext

        validTestAccountToken = InstrumentationRegistry.getArguments()
            .getRequiredArgument(VALID_TEST_ACCOUNT_TOKEN_ARGUMENT_KEY)
        invalidTestAccountToken = InstrumentationRegistry.getArguments()
            .getRequiredArgument(INVALID_TEST_ACCOUNT_TOKEN_ARGUMENT_KEY)

        app = AppInteractor(
            device,
            targetContext,
            validTestAccountToken,
            invalidTestAccountToken
        )

        web = WebViewInteractor(
            targetContext,
            device
        )
    }
}
