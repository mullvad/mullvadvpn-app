package net.mullvad.mullvadvpn.applist

import android.content.pm.PackageManager
import android.graphics.drawable.Drawable
import android.os.Looper
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import io.mockk.verify
import kotlin.test.assertEquals
import kotlin.test.assertFails
import org.junit.After
import org.junit.Before
import org.junit.Test

class ApplicationsIconManagerTest {
    private val mockedPackageManager = mockk<PackageManager>()
    private val mockedMainLooper = mockk<Looper>()
    private val testSubject = ApplicationsIconManager(mockedPackageManager)

    @Before
    fun setUp() {
        mockkStatic(Looper::class)
        every { Looper.getMainLooper() } returns mockedMainLooper
    }

    @After
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun test_first_time_load_icon_from_PM() {
        val testPackageName = "test"
        val mockedDrawable = mockk<Drawable>()
        every { mockedPackageManager.getApplicationIcon(testPackageName) } returns mockedDrawable
        every { mockedMainLooper.isCurrentThread } returns false

        val result = testSubject.getAppIcon(testPackageName)

        assertEquals(mockedDrawable, result)
        verify {
            mockedMainLooper.isCurrentThread
            mockedPackageManager.getApplicationIcon(testPackageName)
        }
    }

    @Test
    fun test_second_time_load_icon_from_cache() {
        val testPackageName = "test"
        val mockedDrawable = mockk<Drawable>()
        every { mockedPackageManager.getApplicationIcon(testPackageName) } returns mockedDrawable
        every { mockedMainLooper.isCurrentThread } returns false

        val result = testSubject.getAppIcon(testPackageName)
        val result2 = testSubject.getAppIcon(testPackageName)

        assertEquals(mockedDrawable, result)
        assertEquals(mockedDrawable, result2)
        verify(exactly = 2) {
            mockedMainLooper.isCurrentThread
        }
        verify(exactly = 1) {
            mockedPackageManager.getApplicationIcon(testPackageName)
        }
    }

    @Test
    fun test_second_time_load_icon_from_PM_after_clear() {
        val testPackageName = "test"
        val mockedDrawable = mockk<Drawable>()
        every { mockedPackageManager.getApplicationIcon(testPackageName) } returns mockedDrawable
        every { mockedMainLooper.isCurrentThread } returns false

        val result = testSubject.getAppIcon(testPackageName)
        testSubject.dispose()
        val result2 = testSubject.getAppIcon(testPackageName)

        assertEquals(mockedDrawable, result)
        assertEquals(mockedDrawable, result2)
        verify(exactly = 2) {
            mockedMainLooper.isCurrentThread
            mockedPackageManager.getApplicationIcon(testPackageName)
        }
    }

    @Test
    fun throw_exception_when_invoke_from_MainThread() {
        val testPackageName = "test"
        every { mockedMainLooper.isCurrentThread } returns true

        assertFails("Should not be called from MainThread") {
            testSubject.getAppIcon(testPackageName)
        }
        verify { mockedMainLooper.isCurrentThread }
    }
}
