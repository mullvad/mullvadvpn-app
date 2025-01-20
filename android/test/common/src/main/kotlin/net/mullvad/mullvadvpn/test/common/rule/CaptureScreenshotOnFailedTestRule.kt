package net.mullvad.mullvadvpn.test.common.rule

import android.content.ContentResolver
import android.content.ContentValues
import android.graphics.Bitmap
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import androidx.annotation.RequiresApi
import androidx.test.platform.app.InstrumentationRegistry.getInstrumentation
import co.touchlab.kermit.Logger
import java.io.File
import java.io.FileOutputStream
import java.io.IOException
import java.nio.file.Paths
import java.time.OffsetDateTime
import java.time.temporal.ChronoUnit
import net.mullvad.mullvadvpn.test.common.misc.Attachment
import org.junit.jupiter.api.extension.ExtensionContext
import org.junit.jupiter.api.extension.TestWatcher

class CaptureScreenshotOnFailedTestRule(private val testTag: String) : TestWatcher {

    override fun testFailed(context: ExtensionContext, cause: Throwable) {
        Logger.d("Capturing screenshot of failed test: " + context.requiredTestMethod.name)
        val timestamp = OffsetDateTime.now().truncatedTo(ChronoUnit.MILLIS)
        val screenshotName = "$timestamp-${context.requiredTestMethod.name}.jpeg"
        captureScreenshot(testTag, screenshotName)
    }

    private fun captureScreenshot(baseDir: String, filename: String) {
        getInstrumentation().uiAutomation.takeScreenshot().apply {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val contentResolver =
                    getInstrumentation().targetContext.applicationContext.contentResolver
                val contentValues = createBaseScreenshotContentValues()
                writeToMediaStore(
                    contentValues = contentValues,
                    contentResolver = contentResolver,
                    baseDir = baseDir,
                    filename = filename,
                )
            } else {
                writeToExternalStorage(baseDir = baseDir, filename = filename)
            }
        }
    }

    @RequiresApi(29)
    private fun Bitmap.writeToMediaStore(
        contentValues: ContentValues,
        contentResolver: ContentResolver,
        baseDir: String,
        filename: String,
    ) {
        contentValues.apply {
            put(MediaStore.MediaColumns.DISPLAY_NAME, filename)
            put(MediaStore.Images.Media.RELATIVE_PATH, "${Attachment.DIRECTORY_PATH}/$baseDir")
        }

        val uri =
            contentResolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, contentValues)

        if (uri != null) {
            contentResolver.openOutputStream(uri).use {
                try {
                    this.compress(Bitmap.CompressFormat.JPEG, 50, it!!)
                } catch (e: IOException) {
                    Logger.e("Unable to store screenshot: ${e.message}")
                }
            }
            contentResolver.update(uri, contentValues, null, null)
        } else {
            Logger.e("Unable to store screenshot")
        }
    }

    private fun Bitmap.writeToExternalStorage(baseDir: String, filename: String) {
        val screenshotBaseDirectory =
            Paths.get(
                    Environment.getExternalStoragePublicDirectory(Attachment.DIRECTORY_PATH).path,
                    baseDir,
                )
                .toFile()
                .apply {
                    if (exists().not()) {
                        mkdirs()
                    }
                }
        FileOutputStream(File(screenshotBaseDirectory, filename)).use { outputStream ->
            try {
                compress(Bitmap.CompressFormat.JPEG, 50, outputStream)
            } catch (e: IOException) {
                Logger.e("Unable to store screenshot: ${e.message}")
            }
        }
    }

    private fun createBaseScreenshotContentValues() =
        ContentValues().apply {
            put(MediaStore.Images.Media.MIME_TYPE, "image/jpeg")
            put(MediaStore.Images.Media.DATE_TAKEN, System.currentTimeMillis())
        }
}
