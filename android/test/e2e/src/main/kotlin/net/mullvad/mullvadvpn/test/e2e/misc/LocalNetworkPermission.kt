package net.mullvad.mullvadvpn.test.e2e.misc

import android.Manifest
import android.os.Build
import androidx.test.platform.app.InstrumentationRegistry.getInstrumentation
import org.junit.jupiter.api.extension.BeforeEachCallback
import org.junit.jupiter.api.extension.ExtensionContext

class LocalNetworkPermission : BeforeEachCallback {
    override fun beforeEach(context: ExtensionContext?) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.CINNAMON_BUN) {
            getInstrumentation()
                .uiAutomation
                .grantRuntimePermission(
                    getInstrumentation().targetContext.packageName,
                    Manifest.permission.ACCESS_LOCAL_NETWORK,
                )
        }
    }
}
