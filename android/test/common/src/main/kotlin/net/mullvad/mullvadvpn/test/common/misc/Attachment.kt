package net.mullvad.mullvadvpn.test.common.misc

import android.content.ContentValues
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import androidx.test.platform.app.InstrumentationRegistry.getInstrumentation
import co.touchlab.kermit.Logger
import java.io.File
import java.io.IOException

class Attachment {
    companion object {
        fun saveAttachment(fileName: String, baseDir: String, data: ByteArray) {
            val contentResolver =
                getInstrumentation().targetContext.applicationContext.contentResolver
            val contentValues =
                ContentValues().apply {
                    put(MediaStore.MediaColumns.DISPLAY_NAME, fileName)
                    put(MediaStore.MediaColumns.MIME_TYPE, "application/octet-stream")
                    put(
                        MediaStore.MediaColumns.RELATIVE_PATH,
                        Environment.DIRECTORY_DOWNLOADS + "/$baseDir"
                    )
                }

            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val uri =
                    contentResolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, contentValues)
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
                val directory =
                    Environment.getExternalStoragePublicDirectory(
                        Environment.DIRECTORY_DOWNLOADS + "/$baseDir"
                    )
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
    }
}
