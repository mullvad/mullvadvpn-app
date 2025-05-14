package net.mullvad.mullvadvpn.test.e2e

import android.Manifest
import android.content.Context
import android.os.Build
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.UiDevice
import co.touchlab.kermit.Logger
import de.mannodermaus.junit5.extensions.GrantPermissionExtension
import net.mullvad.mullvadvpn.test.common.interactor.AppInteractor
import net.mullvad.mullvadvpn.test.common.misc.CaptureScreenRecordingsExtension
import net.mullvad.mullvadvpn.test.common.rule.CaptureScreenshotOnFailedTestRule
import net.mullvad.mullvadvpn.test.e2e.constant.LOG_TAG
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.extension.ExtendWith
import org.junit.jupiter.api.extension.RegisterExtension

@ExtendWith(CaptureScreenRecordingsExtension::class)
abstract class EndToEndTest {

    @RegisterExtension @JvmField val rule = CaptureScreenshotOnFailedTestRule(LOG_TAG)

    @JvmField
    @RegisterExtension
    val extension =
        (if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) {
            GrantPermissionExtension.grant(
                Manifest.permission.WRITE_EXTERNAL_STORAGE,
                Manifest.permission.READ_EXTERNAL_STORAGE,
            )
        } else {
            GrantPermissionExtension.grant()
        })

    lateinit var device: UiDevice
    lateinit var targetContext: Context
    lateinit var app: AppInteractor

    @BeforeEach
    fun setup() {
        Logger.setTag(LOG_TAG)

        device = UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())
        targetContext = InstrumentationRegistry.getInstrumentation().targetContext

        app = AppInteractor(device, targetContext)
    }

    companion object {
        const val DEFAULT_COUNTRY = "Relay Software Country"
        const val DEFAULT_CITY = "Relay Software city"
        const val DEFAULT_RELAY = "se-got-wg-002"

        const val DAITA_COMPATIBLE_COUNTRY = "Relay Software Country"
        const val DAITA_COMPATIBLE_CITY = "Relay Software city"
        const val DAITA_COMPATIBLE_RELAY = "se-got-wg-002"
    }
}
