package net.mullvad.mullvadvpn.test.benchmark

import android.Manifest.permission.READ_EXTERNAL_STORAGE
import android.Manifest.permission.WRITE_EXTERNAL_STORAGE
import android.content.Context
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.UiDevice
import co.touchlab.kermit.Logger
import de.mannodermaus.junit5.extensions.GrantPermissionExtension
import net.mullvad.mullvadvpn.test.benchmark.constant.LOG_TAG
import net.mullvad.mullvadvpn.test.common.interactor.AppInteractor
import net.mullvad.mullvadvpn.test.common.rule.CaptureScreenshotOnFailedTestRule
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.extension.RegisterExtension

abstract class BenchmarkTest {

    @RegisterExtension @JvmField val rule = CaptureScreenshotOnFailedTestRule(LOG_TAG)

    @RegisterExtension
    @JvmField
    val permissionRule: GrantPermissionExtension =
        GrantPermissionExtension.grant(WRITE_EXTERNAL_STORAGE, READ_EXTERNAL_STORAGE)

    lateinit var device: UiDevice
    lateinit var context: Context
    lateinit var targetContext: Context
    lateinit var app: AppInteractor

    @BeforeEach
    open fun setup() {
        Logger.setTag(LOG_TAG)

        device = UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())
        context = InstrumentationRegistry.getInstrumentation().context
        targetContext = InstrumentationRegistry.getInstrumentation().targetContext

        app = AppInteractor(device, targetContext)
    }

    @AfterEach open fun teardown() {}
}
