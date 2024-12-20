package net.mullvad.mullvadvpn.test.common.misc

import android.os.Environment
import co.touchlab.kermit.Logger
import java.io.File
import java.io.IOException
import org.junit.jupiter.api.fail

object Attachment {
    val DIRECTORY_PATH = "${Environment.DIRECTORY_DOWNLOADS}/test-outputs/attachments"

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
