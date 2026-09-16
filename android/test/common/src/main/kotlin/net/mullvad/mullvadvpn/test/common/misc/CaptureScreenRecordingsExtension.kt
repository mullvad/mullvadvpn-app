package net.mullvad.mullvadvpn.test.common.misc

import android.os.Environment
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.platform.app.InstrumentationRegistry.getInstrumentation
import androidx.test.uiautomator.UiDevice
import co.touchlab.kermit.Logger
import java.io.File
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.fail
import org.junit.jupiter.api.extension.AfterEachCallback
import org.junit.jupiter.api.extension.BeforeEachCallback
import org.junit.jupiter.api.extension.ExtensionContext

class CaptureScreenRecordingsExtension : BeforeEachCallback, AfterEachCallback {
    private lateinit var job: Job
    private val coroutineScope = CoroutineScope(Dispatchers.IO)
    private lateinit var device: UiDevice

    override fun beforeEach(context: ExtensionContext?) {
        device = UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())
        val fileName = context?.fileName() ?: "unknown.mp4"
        Logger.v("Starting screen recording. Saving to $fileName")
        startScreenRecord(fileName)
    }

    override fun afterEach(context: ExtensionContext?) {
        Logger.v("Stopping screen recording")
        stopScreenRecord()

        // Delete the recording if the test passed, since it is not required and should not be
        // uploaded to GitHub.
        if (context?.executionException?.isEmpty == true) {
            val fileName = context.fileName()

            Logger.v("Deleting screen recording $fileName")

            val file = File(OUTPUT_DIRECTORY, fileName)
            if (file.exists()) {
                executeShellCommand("rm -f $OUTPUT_DIRECTORY/$fileName")
            } else {
                Logger.w("File $OUTPUT_DIRECTORY/$fileName does not exist")
            }
        }
    }

    private fun startScreenRecord(fileName: String) {
        if (!File(OUTPUT_DIRECTORY).exists()) {
            File(OUTPUT_DIRECTORY).mkdirs()
        }

        job = coroutineScope.launch {
            executeShellCommand("screenrecord $OUTPUT_DIRECTORY/$fileName")
        }
    }

    private fun stopScreenRecord() {
        try {
            executeShellCommand("pkill -2 screenrecord")
            runBlocking { job.join() }
        } catch (e: Exception) {
            Logger.e("Failed to stop recording", e)
            fail("Failed to stop screen recording")
        }
    }

    private fun executeShellCommand(command: String) {
        val fd = getInstrumentation().uiAutomation.executeShellCommand(command)
        fd.close()
    }

    private fun ExtensionContext.fileName(): String {
        val testMethodName = this.testMethod!!.get().name
        return "${testMethodName}.mp4"
    }

    companion object {
        val OUTPUT_DIRECTORY =
            "${Environment.getExternalStorageDirectory().path}/Download/test-attachments/video"
    }
}
