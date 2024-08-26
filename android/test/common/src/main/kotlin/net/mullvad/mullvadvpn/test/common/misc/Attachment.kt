package net.mullvad.mullvadvpn.test.common.misc

import android.os.Environment
import androidx.test.platform.app.InstrumentationRegistry.getInstrumentation
import androidx.test.uiautomator.UiDevice
import co.touchlab.kermit.Logger
import java.io.File
import java.io.IOException

class Attachment {
    companion object {
        private const val DIRECTORY_NAME = "test-attachments"

        fun clearAttachmentsDirectory() {
            val directory = testAttachmentsDirectory()

            val device = UiDevice.getInstance(getInstrumentation())
            device.executeShellCommand("rm -rf ${directory.absolutePath}")
            val createdDirectory = directory.mkdirs()
            if (createdDirectory) {
                Logger.v("Created attachments directory")
            }
        }

        fun saveAttachment(fileName: String, data: ByteArray) {
            val directory = testAttachmentsDirectory()

            val file = File(directory, fileName)
            try {
                file.writeBytes(data)
                Logger.v("Saved attachment ${file.absolutePath}")
            } catch (e: IOException) {
                Logger.e("Failed to save attachment $fileName: ${e.message}")
            }
        }

        private fun testAttachmentsDirectory(): File {
            val externalStorageDirectory = Environment.getExternalStorageDirectory()
            return File(
                externalStorageDirectory,
                Environment.DIRECTORY_DOWNLOADS + "/$DIRECTORY_NAME",
            )
        }
    }
}
