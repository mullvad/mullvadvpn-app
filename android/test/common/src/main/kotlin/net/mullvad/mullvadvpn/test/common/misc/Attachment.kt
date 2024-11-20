package net.mullvad.mullvadvpn.test.common.misc

import android.os.Environment
import co.touchlab.kermit.Logger
import java.io.File
import java.io.IOException
import org.junit.jupiter.api.fail

object Attachment {
    const val DIRECTORY_NAME = "test-outputs"
    val DIRECTORY_PATH = "${Environment.DIRECTORY_DOWNLOADS}/${DIRECTORY_NAME}"

    private val testAttachmentsDirectory =
        File(
            Environment.getExternalStorageDirectory(),
            DIRECTORY_PATH,
        )

    fun saveAttachment(fileName: String, data: ByteArray) {
        createAttachmentsDirectoryIfNotExists()

        val file = File(testAttachmentsDirectory, fileName)
        try {
            file.writeBytes(data)
            Logger.v("Saved attachment ${file.absolutePath}")
        } catch (e: IOException) {
            fail("Failed to save attachment $fileName: ${e.message}")
        }
    }

    private fun createAttachmentsDirectoryIfNotExists() {
        if (!testAttachmentsDirectory.exists() && !testAttachmentsDirectory.mkdirs()) {
            fail("Failed to create directory ${testAttachmentsDirectory.absolutePath}")
        }
    }
}
