package net.mullvad.mullvadvpn.applist

import android.Manifest
import android.content.pm.ApplicationInfo
import android.content.pm.PackageManager
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import io.mockk.verifyAll
import net.mullvad.mullvadvpn.assertLists
import org.junit.After
import org.junit.Test

class ApplicationsProviderTest {
    private val mockedPackageManager = mockk<PackageManager>()
    private val selfPackageName = "self_package_name"
    private val testSubject = ApplicationsProvider(mockedPackageManager, selfPackageName)
    private val internet = Manifest.permission.INTERNET

    @After
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun test_get_apps() {
        val launchWithInternetPackageName = "launch_with_internet_package_name"
        val launchWithoutInternetPackageName = "launch_without_internet_package_name"
        val nonLaunchWithInternetPackageName = "non_launch_with_internet_package_name"
        val nonLaunchWithoutInternetPackageName = "non_launch_without_internet_package_name"

        every {
            mockedPackageManager.getInstalledApplications(PackageManager.GET_META_DATA)
        } returns listOf(
            createApplicationInfo(launchWithInternetPackageName, launch = true, internet = true),
            createApplicationInfo(launchWithoutInternetPackageName, launch = true),
            createApplicationInfo(nonLaunchWithInternetPackageName, internet = true),
            createApplicationInfo(nonLaunchWithoutInternetPackageName),
            createApplicationInfo(selfPackageName, internet = true, launch = true)
        )

        val result = testSubject.getAppsList()
        val expected = listOf(
            AppData(launchWithInternetPackageName, 0, launchWithInternetPackageName)
        )

        assertLists(expected, result)

        verifyAll {
            mockedPackageManager.getInstalledApplications(PackageManager.GET_META_DATA)

            mockedPackageManager.checkPermission(internet, launchWithInternetPackageName)
            mockedPackageManager.checkPermission(internet, launchWithoutInternetPackageName)
            mockedPackageManager.checkPermission(internet, nonLaunchWithInternetPackageName)
            mockedPackageManager.checkPermission(internet, nonLaunchWithoutInternetPackageName)
            mockedPackageManager.checkPermission(internet, selfPackageName)

            mockedPackageManager.getLaunchIntentForPackage(launchWithInternetPackageName)
            mockedPackageManager.getLaunchIntentForPackage(nonLaunchWithInternetPackageName)
            mockedPackageManager.getLaunchIntentForPackage(selfPackageName)
        }
    }

    private fun createApplicationInfo(
        packageName: String,
        launch: Boolean = false,
        internet: Boolean = false
    ): ApplicationInfo {
        val mockApplicationInfo = mockk<ApplicationInfo>()

        mockApplicationInfo.packageName = packageName
        mockApplicationInfo.icon = 0

        every { mockApplicationInfo.loadLabel(mockedPackageManager) } returns packageName

        every {
            mockedPackageManager.getLaunchIntentForPackage(packageName)
        } returns if (launch)
            mockk()
        else
            null

        every {
            mockedPackageManager.checkPermission(Manifest.permission.INTERNET, packageName)
        } returns if (internet)
            PackageManager.PERMISSION_GRANTED
        else
            PackageManager.PERMISSION_DENIED

        return mockApplicationInfo
    }
}
