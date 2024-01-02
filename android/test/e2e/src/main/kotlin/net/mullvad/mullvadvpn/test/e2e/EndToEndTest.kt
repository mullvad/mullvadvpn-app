package net.mullvad.mullvadvpn.test.e2e

import android.Manifest
import android.content.Context
import android.os.Build
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.UiDevice
import de.mannodermaus.junit5.extensions.GrantPermissionExtension
import net.mullvad.mullvadvpn.test.common.interactor.AppInteractor
import net.mullvad.mullvadvpn.test.common.rule.CaptureScreenshotOnFailedTestRule
import net.mullvad.mullvadvpn.test.e2e.constant.INVALID_TEST_ACCOUNT_TOKEN_ARGUMENT_KEY
import net.mullvad.mullvadvpn.test.e2e.constant.LOG_TAG
import net.mullvad.mullvadvpn.test.e2e.constant.VALID_TEST_ACCOUNT_TOKEN_ARGUMENT_KEY
import net.mullvad.mullvadvpn.test.e2e.extension.getRequiredArgument
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.extension.RegisterExtension

abstract class EndToEndTest {

    @RegisterExtension @JvmField val rule = CaptureScreenshotOnFailedTestRule(LOG_TAG)

    @JvmField
    @RegisterExtension
    val extension =
        (if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) {
            GrantPermissionExtension.grant(
                Manifest.permission.WRITE_EXTERNAL_STORAGE,
                Manifest.permission.READ_EXTERNAL_STORAGE
            )
        } else {
            null
        })

    lateinit var device: UiDevice
    lateinit var targetContext: Context
    lateinit var app: AppInteractor
    lateinit var validTestAccountToken: String
    lateinit var invalidTestAccountToken: String

    @BeforeEach
    fun setup() {
        device = UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())
        targetContext = InstrumentationRegistry.getInstrumentation().targetContext

        validTestAccountToken =
            InstrumentationRegistry.getArguments()
                .getRequiredArgument(VALID_TEST_ACCOUNT_TOKEN_ARGUMENT_KEY)
        invalidTestAccountToken =
            InstrumentationRegistry.getArguments()
                .getRequiredArgument(INVALID_TEST_ACCOUNT_TOKEN_ARGUMENT_KEY)

        app = AppInteractor(device, targetContext)
    }
}
