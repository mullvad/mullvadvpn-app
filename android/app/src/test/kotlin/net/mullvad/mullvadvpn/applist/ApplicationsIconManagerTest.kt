package net.mullvad.mullvadvpn.applist

import android.content.pm.PackageManager
import android.graphics.Bitmap
import android.graphics.drawable.Drawable
import android.os.Looper
import androidx.core.graphics.drawable.toBitmap
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import io.mockk.verify
import kotlin.test.assertEquals
import kotlin.test.assertFails
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class ApplicationsIconManagerTest {
    private val mockedPackageManager = mockk<PackageManager>()
    private val mockedMainLooper = mockk<Looper>()
    private val testSubject = ApplicationsIconManager(mockedPackageManager)

    @BeforeEach
    fun setUp() {
        mockkStatic(Looper::class)
        mockkStatic(DRAWABLE_EXTENSION_CLASS)
        every { Looper.getMainLooper() } returns mockedMainLooper
    }

    @AfterEach
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun test_first_time_load_icon_from_PM() {
        val testPackageName = "test"
        val mockedBitmap = mockk<Bitmap>()
        val mockedDrawable = mockk<Drawable>().apply { every { toBitmap() } returns mockedBitmap }
        every { mockedPackageManager.getApplicationIcon(testPackageName) } returns mockedDrawable
        every { mockedMainLooper.isCurrentThread } returns false
        every { mockedDrawable.intrinsicWidth } returns 0
        every { mockedDrawable.intrinsicHeight } returns 0

        val result = testSubject.getAppIcon(testPackageName)

        assertEquals(mockedBitmap, result)
        verify {
            mockedMainLooper.isCurrentThread
            mockedPackageManager.getApplicationIcon(testPackageName)
        }
    }

    @Test
    fun test_second_time_load_icon_from_cache() {
        val testPackageName = "test"
        val mockedBitmap = mockk<Bitmap>()
        val mockedDrawable = mockk<Drawable>().apply { every { toBitmap() } returns mockedBitmap }
        every { mockedPackageManager.getApplicationIcon(testPackageName) } returns mockedDrawable
        every { mockedMainLooper.isCurrentThread } returns false
        every { mockedDrawable.intrinsicWidth } returns 0
        every { mockedDrawable.intrinsicHeight } returns 0

        val result = testSubject.getAppIcon(testPackageName)
        val result2 = testSubject.getAppIcon(testPackageName)

        assertEquals(mockedBitmap, result)
        assertEquals(mockedBitmap, result2)
        verify(exactly = 2) { mockedMainLooper.isCurrentThread }
        verify(exactly = 1) { mockedPackageManager.getApplicationIcon(testPackageName) }
    }

    @Test
    fun test_second_time_load_icon_from_PM_after_clear() {
        val testPackageName = "test"
        val mockedBitmap = mockk<Bitmap>()
        val mockedDrawable = mockk<Drawable>().apply { every { toBitmap() } returns mockedBitmap }
        every { mockedPackageManager.getApplicationIcon(testPackageName) } returns mockedDrawable
        every { mockedMainLooper.isCurrentThread } returns false
        every { mockedDrawable.intrinsicWidth } returns 0
        every { mockedDrawable.intrinsicHeight } returns 0

        val result = testSubject.getAppIcon(testPackageName)
        testSubject.dispose()
        val result2 = testSubject.getAppIcon(testPackageName)

        assertEquals(mockedBitmap, result)
        assertEquals(mockedBitmap, result2)
        verify(exactly = 2) {
            mockedMainLooper.isCurrentThread
            mockedPackageManager.getApplicationIcon(testPackageName)
        }
    }

    @Test
    fun test_throw_exception_when_invoke_from_MainThread() {
        val testPackageName = "test"
        every { mockedMainLooper.isCurrentThread } returns true

        assertFails("Should not be called from MainThread") {
            testSubject.getAppIcon(testPackageName)
        }
        verify { mockedMainLooper.isCurrentThread }
    }

    companion object {
        private const val DRAWABLE_EXTENSION_CLASS = "androidx.core.graphics.drawable.DrawableKt"
    }
}
