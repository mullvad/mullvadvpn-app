package net.mullvad.mullvadvpn.test.common.misc

import android.os.Environment
import co.touchlab.kermit.Logger
import java.io.File
import java.io.IOException
import org.junit.jupiter.api.fail

object Attachment {
    private const val DIRECTORY_NAME = "test-attachments"
    public val testAttachmentsDirectory =
        File(
            Environment.getExternalStorageDirectory(),
            Environment.DIRECTORY_DOWNLOADS + "/$DIRECTORY_NAME",
        )

    fun saveAttachment(fileName: String, data: ByteArray) {
        if (testAttachmentsDirectory.exists().not()) {
            if (testAttachmentsDirectory.mkdirs().not()) {
                fail("Failed to create directory ${testAttachmentsDirectory.absolutePath}")
            }
        }

        val file = File(testAttachmentsDirectory, fileName)
        try {
            file.writeBytes(data)
            Logger.v("Saved attachment ${file.absolutePath}")
        } catch (e: IOException) {
            fail("Failed to save attachment $fileName: ${e.message}")
        }
    }
}
