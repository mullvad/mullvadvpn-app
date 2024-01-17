package net.mullvad.mullvadvpn.test.e2e

import android.Manifest
import android.content.Context
import android.os.Build
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.UiDevice
import de.mannodermaus.junit5.extensions.GrantPermissionExtension
import net.mullvad.mullvadvpn.test.common.interactor.AppInteractor
import net.mullvad.mullvadvpn.test.common.rule.CaptureScreenshotOnFailedTestRule
import net.mullvad.mullvadvpn.test.e2e.constant.LOG_TAG
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.extension.RegisterExtension

abstract class EndToEndTest(private val infra: String) {

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
            GrantPermissionExtension.grant()
        })

    lateinit var device: UiDevice
    lateinit var targetContext: Context
    lateinit var app: AppInteractor

    @BeforeEach
    fun setup() {
        device = UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())
        targetContext = InstrumentationRegistry.getInstrumentation().targetContext

        val targetPackageNameSuffix =
            when (infra) {
                "devmole" -> ".devmole"
                "stagemole" -> ".stagemole"
                else -> ""
            }

        app = AppInteractor(device, targetContext, "net.mullvad.mullvadvpn$targetPackageNameSuffix")
    }
}
