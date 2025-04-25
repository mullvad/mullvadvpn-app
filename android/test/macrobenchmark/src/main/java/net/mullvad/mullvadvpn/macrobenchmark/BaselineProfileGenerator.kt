package net.mullvad.mullvadvpn.macrobenchmark

import android.os.Build
import androidx.annotation.RequiresApi
import androidx.benchmark.macro.junit4.BaselineProfileRule
import org.junit.Rule
import org.junit.Test

// @OptIn(ExperimentalBaselineProfilesApi::class)
@RequiresApi(Build.VERSION_CODES.P)
class BaselineProfileGenerator {
    @get:Rule val baselineProfileRule = BaselineProfileRule()

    @Test
    fun appStartupAndUserJourneys() {
        baselineProfileRule.collect(packageName = "net.mullvad.mullvadvpn") {
            // App startup journey.
            startActivityAndWait()

            //            device.findObject(By.text("COMPOSE
            // LAZYLIST")).clickAndWait(Until.newWindow(), 1_000)
            //            device.findObject(By.res("myLazyColumn")).also {
            //                it.fling(Direction.DOWN)
            //                it.fling(Direction.UP)
            //            }
            //            device.pressBack()
        }
    }
}
