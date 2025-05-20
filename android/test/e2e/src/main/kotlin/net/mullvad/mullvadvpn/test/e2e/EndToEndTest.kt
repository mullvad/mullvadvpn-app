package net.mullvad.mullvadvpn.test.e2e

import android.Manifest
import android.app.Activity
import android.app.Application
import android.os.Build
import android.os.Bundle
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.UiDevice
import co.touchlab.kermit.Logger
import de.mannodermaus.junit5.extensions.GrantPermissionExtension
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.test.common.interactor.AppInteractor
import net.mullvad.mullvadvpn.test.common.misc.ActivityLifecycleCallbacksAdapter
import net.mullvad.mullvadvpn.test.common.misc.CaptureScreenRecordingsExtension
import net.mullvad.mullvadvpn.test.common.rule.CaptureScreenshotOnFailedTestRule
import net.mullvad.mullvadvpn.test.e2e.constant.LOG_TAG
import net.mullvad.mullvadvpn.test.e2e.router.firewall.FirewallClient
import org.junit.jupiter.api.BeforeAll
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
    lateinit var targetApplication: Application
    lateinit var targetActivity: Activity
    lateinit var app: AppInteractor

    @BeforeEach
    fun setup() {
        Logger.setTag(LOG_TAG)

        device = UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())
        targetApplication =
            InstrumentationRegistry.getInstrumentation().targetContext.applicationContext
                as Application

        app = AppInteractor(device, targetApplication)

        registerActivityLifecycleCallbacks(targetApplication)
    }

    private fun registerActivityLifecycleCallbacks(app: Application) =
        app.registerActivityLifecycleCallbacks(
            object : ActivityLifecycleCallbacksAdapter() {
                override fun onActivityCreated(activity: Activity, savedInstanceState: Bundle?) {
                    Logger.d("onActivityCreated")
                    targetActivity = activity
                }
            }
        )

    companion object {
        val firewallClient = FirewallClient()

        @JvmStatic
        @BeforeAll
        // There are certain scenarios where old rules from previous tests runs may remain on
        // the router and cause issues, so attempt to clear them before any test setup is done.
        fun clearFirewallRules() {
            runBlocking {
                try {
                    firewallClient.removeAllRules()
                } catch (e: Exception) {
                    // If the router can't be reached we ignore the error because the e2e
                    // test that is about to be run may not require router access.
                    Logger.e("firewallClient.removeAllRules() failed")
                }
            }
        }
    }
}
