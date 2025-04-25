package net.mullvad.mullvadvpn.macrobenchmark

import androidx.benchmark.macro.BaselineProfileMode
import androidx.benchmark.macro.CompilationMode
import androidx.benchmark.macro.StartupMode
import androidx.benchmark.macro.StartupTimingMetric
import androidx.benchmark.macro.junit4.MacrobenchmarkRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.LargeTest
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * This is an example startup benchmark.
 *
 * It navigates to the device's home screen, and launches the default activity.
 *
 * Before running this benchmark:
 * 1) switch your app's active build variant in the Studio (affects Studio runs only)
 * 2) add `<profileable android:shell="true" />` to your app's manifest, within the `<application>`
 *    tag
 *
 * Run this benchmark from Studio to see startup measurements, and captured system traces for
 * investigating your app's performance.
 */
// @RunWith(AndroidJUnit4::class)
// class StartupBenchmark {
//    @get:Rule val benchmarkRule = MacrobenchmarkRule()
//
//    @Test
//    fun startup() =
//        benchmarkRule.measureRepeated(
//            packageName = "net.mullvad.mullvadvpn",
//            metrics = listOf(StartupTimingMetric()),
//            iterations = 5,
//            startupMode = StartupMode.COLD,
//            setupBlock = {
//                // Press home button before each run to ensure the starting activity isn't
// visible.
//                pressHome(1000)
//            },
//        ) {
//            // starts default launch activity
//            startActivityAndWait()
//        }
// }

@RunWith(AndroidJUnit4::class)
@LargeTest
class StartupBenchmark {

    @get:Rule val rule = MacrobenchmarkRule()

    @Test fun startupCompilationNone() = benchmark(CompilationMode.None())

    @Test
    fun startupCompilationBaselineProfiles() =
        benchmark(CompilationMode.Partial(BaselineProfileMode.Require))

    private fun benchmark(compilationMode: CompilationMode) {
        // The application id for the running build variant is read from the instrumentation
        // arguments.
        rule.measureRepeated(
            packageName = "net.mullvad.mullvadvpn",
            metrics = listOf(StartupTimingMetric()),
            compilationMode = compilationMode,
            startupMode = StartupMode.COLD,
            iterations = 10,
            setupBlock = { pressHome(1000) },
            measureBlock = {
                startActivityAndWait()

                // TODO Add interactions to wait for when your app is fully drawn.
                // The app is fully drawn when Activity.reportFullyDrawn is called.
                // For Jetpack Compose, you can use ReportDrawn, ReportDrawnWhen and
                // ReportDrawnAfter
                // from the AndroidX Activity library.

                // Check the UiAutomator documentation for more information on how to
                // interact with the app.
                // https://d.android.com/training/testing/other-components/ui-automator
            },
        )
    }
}
