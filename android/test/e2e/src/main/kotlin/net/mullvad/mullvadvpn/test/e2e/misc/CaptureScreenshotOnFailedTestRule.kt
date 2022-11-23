package net.mullvad.mullvadvpn.test.e2e.misc

import android.util.Log
import androidx.test.runner.screenshot.BasicScreenCaptureProcessor
import androidx.test.runner.screenshot.ScreenCaptureProcessor
import androidx.test.runner.screenshot.Screenshot
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter
import net.mullvad.mullvadvpn.test.e2e.constant.LOG_TAG
import org.junit.rules.TestWatcher
import org.junit.runner.Description

class CaptureScreenshotOnFailedTestRule : TestWatcher() {
    override fun failed(e: Throwable?, description: Description?) {
        Log.d(LOG_TAG, "Capturing screenshot of failed test: " + description?.methodName)
        val timestamp = DateTimeFormatter.ISO_DATE_TIME.format(LocalDateTime.now()).replace(":", "")
        val screenshotName = "$timestamp-${description?.methodName}"
        captureScreenshot(screenshotName)
    }

    private fun captureScreenshot(screenShotName: String) {
        try {
            val screenCapture = Screenshot.capture().apply { name = screenShotName }
            val processorSet: MutableSet<ScreenCaptureProcessor> = HashSet()
            processorSet.add(BasicScreenCaptureProcessor())
            screenCapture.process(processorSet)
        } catch (ex: Exception) {
            Log.d(LOG_TAG, "Error capturing screenshot: " + ex.message)
        }
    }
}
