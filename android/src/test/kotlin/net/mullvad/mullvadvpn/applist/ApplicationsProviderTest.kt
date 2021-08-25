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
        val leanbackLaunchWithInternetPackageName = "leanback_launch_with_internet_package_name"
        val leanbackLaunchWithoutInternetPackageName =
            "leanback_launch_without_internet_package_name"

        every {
            mockedPackageManager.getInstalledApplications(PackageManager.GET_META_DATA)
        } returns listOf(
            createApplicationInfo(launchWithInternetPackageName, launch = true, internet = true),
            createApplicationInfo(launchWithoutInternetPackageName, launch = true),
            createApplicationInfo(nonLaunchWithInternetPackageName, internet = true),
            createApplicationInfo(nonLaunchWithoutInternetPackageName),
            createApplicationInfo(
                leanbackLaunchWithInternetPackageName,
                leanback = true,
                internet = true
            ),
            createApplicationInfo(leanbackLaunchWithoutInternetPackageName, leanback = true),
            createApplicationInfo(selfPackageName, internet = true, launch = true)
        )

        val result = testSubject.getAppsList()
        val expected = listOf(
            AppData(launchWithInternetPackageName, 0, launchWithInternetPackageName),
            AppData(nonLaunchWithInternetPackageName, 0, nonLaunchWithInternetPackageName, true),
            AppData(leanbackLaunchWithInternetPackageName, 0, leanbackLaunchWithInternetPackageName)
        )

        assertLists(expected, result)

        verifyAll {
            mockedPackageManager.getInstalledApplications(PackageManager.GET_META_DATA)

            listOf(
                launchWithInternetPackageName,
                launchWithoutInternetPackageName,
                nonLaunchWithInternetPackageName,
                nonLaunchWithoutInternetPackageName,
                leanbackLaunchWithInternetPackageName,
                leanbackLaunchWithoutInternetPackageName,
                selfPackageName
            ).forEach { packageName ->
                mockedPackageManager.checkPermission(internet, packageName)
            }

            listOf(
                launchWithInternetPackageName,
                nonLaunchWithInternetPackageName,
                leanbackLaunchWithInternetPackageName
            ).forEach { packageName ->
                mockedPackageManager.getLaunchIntentForPackage(packageName)
            }

            listOf(
                nonLaunchWithInternetPackageName,
                leanbackLaunchWithInternetPackageName,
            ).forEach { packageName ->
                mockedPackageManager.getLeanbackLaunchIntentForPackage(packageName)
            }
        }
    }

    private fun createApplicationInfo(
        packageName: String,
        launch: Boolean = false,
        leanback: Boolean = false,
        internet: Boolean = false,
        systemApp: Boolean = false
    ): ApplicationInfo {
        val mockApplicationInfo = mockk<ApplicationInfo>()

        mockApplicationInfo.packageName = packageName
        mockApplicationInfo.icon = 0

        every { mockApplicationInfo.loadLabel(mockedPackageManager) } returns packageName

        every {
            mockedPackageManager.getLaunchIntentForPackage(packageName)
        } returns if (launch || systemApp)
            mockk()
        else
            null

        every {
            mockedPackageManager.getLeanbackLaunchIntentForPackage(packageName)
        } returns if (leanback || systemApp)
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
