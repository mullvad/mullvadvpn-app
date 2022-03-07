package net.mullvad.mullvadvpn.e2e

import android.content.Context
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.runner.AndroidJUnit4
import androidx.test.uiautomator.UiDevice
import net.mullvad.mullvadvpn.e2e.interactor.AppInteractor
import net.mullvad.mullvadvpn.e2e.misc.CaptureScreenshotOnFailedTestRule
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

    @Before
    fun setup() {
        device = UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())
        targetContext = InstrumentationRegistry.getInstrumentation().targetContext

        app = AppInteractor(
            device,
            targetContext
        )
    }
}
