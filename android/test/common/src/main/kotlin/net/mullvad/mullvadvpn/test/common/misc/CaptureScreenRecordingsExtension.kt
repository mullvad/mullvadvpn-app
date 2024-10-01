package net.mullvad.mullvadvpn.test.common.misc

import android.os.Environment
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.UiDevice
import co.touchlab.kermit.Logger
import java.io.File
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.fail
import org.junit.jupiter.api.extension.AfterEachCallback
import org.junit.jupiter.api.extension.BeforeEachCallback
import org.junit.jupiter.api.extension.ExtensionContext

class CaptureScreenRecordingsExtension : BeforeEachCallback, AfterEachCallback {
    private lateinit var job: Job

    override fun beforeEach(context: ExtensionContext?) {
        val testMethodName = context?.testMethod!!.get().name
        val fileName = "${testMethodName}.mp4"
        Logger.v("Starting screen recording. Saving to $testMethodName")
        startScreenRecord(fileName)
    }

    override fun afterEach(context: ExtensionContext?) {
        Logger.v("Stopping screen recording")
        stopScreenRecord()
    }

    private fun startScreenRecord(fileName: String) {
        if (File(OUTPUT_DIRECTORY).exists().not()) {
            File(OUTPUT_DIRECTORY).mkdirs()
        }

        job =
            GlobalScope.launch {
                UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())
                    .executeShellCommand("screenrecord $OUTPUT_DIRECTORY/$fileName")
            }
    }

    private fun stopScreenRecord() {
        try {
            UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())
                .executeShellCommand("pkill -2 screenrecord")
            runBlocking { job.join() }
        } catch (e: Exception) {
            fail("Failed to stop screen recording")
        }
    }

    companion object {
        val OUTPUT_DIRECTORY =
            "${Environment.getExternalStorageDirectory().path}/Download/test-attachments/video"
    }
}
