package net.mullvad.mullvadvpn.test.common.misc

import android.os.Environment
import androidx.test.platform.app.InstrumentationRegistry
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
            coroutineScope.launch {
                device.executeShellCommand("screenrecord $OUTPUT_DIRECTORY/$fileName")
            }
    }

    private fun stopScreenRecord() {
        try {
            device.executeShellCommand("pkill -2 screenrecord")
            runBlocking { job.join() }
        } catch (e: Exception) {
            fail("Failed to stop screen recording")
        }
    }

    companion object {
        val OUTPUT_DIRECTORY =
            "${Environment.getExternalStorageDirectory().path}/Download/${Attachment.DIRECTORY_NAME}/video"
    }
}
