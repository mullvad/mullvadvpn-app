package net.mullvad.mullvadvpn.test.common.misc

import android.content.ContentValues
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import androidx.test.platform.app.InstrumentationRegistry.getInstrumentation
import co.touchlab.kermit.Logger
import java.io.File
import java.io.IOException

class Attachment {
    companion object {
        private const val DIRECTORY_NAME = "test-attachments"

        fun clearAttachmentsDirectory() {
            val contentResolver =
                getInstrumentation().targetContext.applicationContext.contentResolver
            val directory = testAttachmentsDirectory()

            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                if (directory.exists()) {
                    contentResolver.delete(Uri.fromFile(directory), null, null)
                    Logger.v("Cleared attachments directory")
                }
            } else {
                if (directory.exists()) {
                    directory.delete()
                    Logger.v("Cleared attachments directory")
                }
            }

            if (!directory.exists()) {
                directory.mkdirs()
                Logger.v("Created attachments directory")
            }
        }

        fun saveAttachment(fileName: String, baseDir: String, data: ByteArray) {
            val directory = testAttachmentsDirectory()
            val contentResolver =
                getInstrumentation().targetContext.applicationContext.contentResolver
            val contentValues =
                ContentValues().apply {
                    put(MediaStore.MediaColumns.DISPLAY_NAME, fileName)
                    put(MediaStore.MediaColumns.MIME_TYPE, "application/octet-stream")
                    put(
                        MediaStore.MediaColumns.RELATIVE_PATH,
                        Environment.DIRECTORY_DOWNLOADS + "/$baseDir/$DIRECTORY_NAME",
                    )
                }

            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val uri = Uri.fromFile(testAttachmentsDirectory())
                if (uri != null) {
                    contentResolver.openOutputStream(uri).use { outputStream ->
                        outputStream?.write(data)
                        outputStream?.close()
                        contentResolver.update(uri, contentValues, null, null)

                        Logger.v("Saved attachment ${uri.toString()}")
                    }
                    Logger.v("Saved attachment ${uri.toString()}")
                } else {
                    Logger.e("Failed to save attachment $fileName")
                }
            } else {
                if (!directory.exists()) {
                    directory.mkdirs()
                }

                val file = File(directory, fileName)
                try {
                    file.writeBytes(data)
                    Logger.v("Saved attachment ${file.absolutePath}")
                } catch (e: IOException) {
                    Logger.e("Failed to save attachment $fileName: ${e.message}")
                }
            }
        }

        private fun testAttachmentsDirectory(): File {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val uri =
                    MediaStore.Downloads.EXTERNAL_CONTENT_URI.buildUpon()
                        .appendPath(DIRECTORY_NAME)
                        .build()
                return File(uri.path!!)
            } else {
                return File(
                    Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS),
                    DIRECTORY_NAME,
                )
            }
        }
    }
}
