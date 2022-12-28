package net.mullvad.mullvadvpn.test.e2e

import android.Manifest
import android.content.Context
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.rule.GrantPermissionRule
import androidx.test.runner.AndroidJUnit4
import androidx.test.uiautomator.UiDevice
import net.mullvad.mullvadvpn.test.common.interactor.AppInteractor
import net.mullvad.mullvadvpn.test.common.rule.CaptureScreenshotOnFailedTestRule
import net.mullvad.mullvadvpn.test.e2e.constant.INVALID_TEST_ACCOUNT_TOKEN_ARGUMENT_KEY
import net.mullvad.mullvadvpn.test.e2e.constant.LOG_TAG
import net.mullvad.mullvadvpn.test.e2e.constant.VALID_TEST_ACCOUNT_TOKEN_ARGUMENT_KEY
import net.mullvad.mullvadvpn.test.e2e.extension.getRequiredArgument
import org.junit.Before
import org.junit.Rule
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
abstract class EndToEndTest {

    @Rule
    @JvmField
    val rule = CaptureScreenshotOnFailedTestRule(LOG_TAG)

    @Rule
    @JvmField
    val permissionRule: GrantPermissionRule = GrantPermissionRule.grant(
        Manifest.permission.WRITE_EXTERNAL_STORAGE,
        Manifest.permission.READ_EXTERNAL_STORAGE
    )

    lateinit var device: UiDevice
    lateinit var targetContext: Context
    lateinit var app: AppInteractor
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
            targetContext
        )
    }
}
