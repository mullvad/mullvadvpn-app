package net.mullvad.mullvadvpn.test.common.rule

import android.content.ContentResolver
import android.content.ContentValues
import android.graphics.Bitmap
import android.os.Build
import android.os.Environment
import android.os.Environment.DIRECTORY_PICTURES
import android.provider.MediaStore
import android.util.Log
import androidx.annotation.RequiresApi
import androidx.test.platform.app.InstrumentationRegistry.getInstrumentation
import java.io.File
import java.io.FileOutputStream
import java.io.IOException
import java.nio.file.Paths
import java.time.OffsetDateTime
import java.time.temporal.ChronoUnit
import org.junit.rules.TestWatcher
import org.junit.runner.Description

class CaptureScreenshotOnFailedTestRule(private val testTag: String) : TestWatcher() {

    override fun failed(e: Throwable?, description: Description) {
        Log.d(testTag, "Capturing screenshot of failed test: " + description.methodName)
        val timestamp = OffsetDateTime.now().truncatedTo(ChronoUnit.MILLIS)
        val screenshotName = "$timestamp-${description.methodName}.jpeg"
        captureScreenshot(testTag, screenshotName)
    }

    private fun captureScreenshot(baseDir: String, filename: String) {
        val contentResolver = getInstrumentation().targetContext.applicationContext.contentResolver
        val contentValues = createBaseScreenshotContentValues()

        getInstrumentation().uiAutomation.takeScreenshot().apply {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                writeToMediaStore(
                    contentValues = contentValues,
                    contentResolver = contentResolver,
                    baseDir = baseDir,
                    filename = filename
                )
            } else {
                writeToExternalStorage(
                    contentValues = contentValues,
                    contentResolver = contentResolver,
                    baseDir = baseDir,
                    filename = filename
                )
            }
        }
    }

    @RequiresApi(29)
    private fun Bitmap.writeToMediaStore(
        contentValues: ContentValues,
        contentResolver: ContentResolver,
        baseDir: String,
        filename: String
    ) {
        contentValues.apply {
            put(MediaStore.MediaColumns.DISPLAY_NAME, filename)
            put(
                MediaStore.Images.Media.RELATIVE_PATH,
                "$DIRECTORY_PICTURES/$baseDir"
            )
        }

        val uri =
            contentResolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, contentValues)

        if (uri != null) {
            contentResolver.openOutputStream(uri).use {
                try {
                    this.compress(Bitmap.CompressFormat.JPEG, 50, it)
                } catch (e: IOException) {
                    Log.e(testTag, "Unable to store screenshot: ${e.message}")
                }
            }
            contentResolver.update(uri, contentValues, null, null)
        } else {
            Log.e(testTag, "Unable to store screenshot")
        }
    }

    private fun Bitmap.writeToExternalStorage(
        contentValues: ContentValues,
        contentResolver: ContentResolver,
        baseDir: String,
        filename: String
    ) {
        val screenshotBaseDirectory = Paths.get(
            Environment.getExternalStoragePublicDirectory(DIRECTORY_PICTURES).path,
            baseDir,
        ).toFile().apply {
            if (exists().not()) {
                mkdirs()
            }
        }
        FileOutputStream(File(screenshotBaseDirectory, filename)).use { outputStream ->
            try {
                this.compress(Bitmap.CompressFormat.JPEG, 50, outputStream)
            } catch (e: IOException) {
                Log.e(testTag, "Unable to store screenshot: ${e.message}")
            }
        }
        contentResolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, contentValues)
    }

    private fun createBaseScreenshotContentValues() = ContentValues().apply {
        put(MediaStore.Images.Media.MIME_TYPE, "image/jpeg")
        put(MediaStore.Images.Media.DATE_TAKEN, System.currentTimeMillis())
    }
}
