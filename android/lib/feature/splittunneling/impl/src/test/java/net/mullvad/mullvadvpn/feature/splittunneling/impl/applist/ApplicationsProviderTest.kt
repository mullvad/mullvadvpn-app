package net.mullvad.mullvadvpn.feature.splittunneling.impl.applist

import android.Manifest
import android.annotation.SuppressLint
import android.content.pm.ApplicationInfo
import android.content.pm.PackageManager
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import io.mockk.verifyAll
import kotlin.test.assertEquals
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.model.AppId
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Test

class ApplicationsProviderTest {
    private val mockedPackageManager = mockk<PackageManager>()
    private val selfPackageName = AppId("self_package_name")
    private val testSubject = ApplicationsProvider(mockedPackageManager, selfPackageName.value)

    private val internet = Manifest.permission.INTERNET

    @AfterEach
    fun tearDown() {
        unmockkAll()
    }

    @SuppressLint("UseCheckPermission")
    @Test
    fun `fetch all apps should work`() {
        val launchWithInternetPackageName = AppId("launch_with_internet_package_name")
        val launchWithoutInternetPackageName = AppId("launch_without_internet_package_name")
        val nonLaunchWithInternetPackageName = AppId("non_launch_with_internet_package_name")
        val nonLaunchWithoutInternetPackageName = AppId("non_launch_without_internet_package_name")
        val leanbackLaunchWithInternetPackageName =
            AppId("leanback_launch_with_internet_package_name")
        val leanbackLaunchWithoutInternetPackageName =
            AppId("leanback_launch_without_internet_package_name")

        every {
            mockedPackageManager.getInstalledApplications(PackageManager.GET_META_DATA)
        } returns
            listOf(
                createApplicationInfo(
                    launchWithInternetPackageName,
                    launch = true,
                    internet = true,
                ),
                createApplicationInfo(launchWithoutInternetPackageName, launch = true),
                createApplicationInfo(nonLaunchWithInternetPackageName, internet = true),
                createApplicationInfo(nonLaunchWithoutInternetPackageName),
                createApplicationInfo(
                    leanbackLaunchWithInternetPackageName,
                    leanback = true,
                    internet = true,
                ),
                createApplicationInfo(leanbackLaunchWithoutInternetPackageName, leanback = true),
                createApplicationInfo(selfPackageName, internet = true, launch = true),
            )

        val result = testSubject.apps()
        val expected =
            listOf(
                AppData(launchWithInternetPackageName, 0, launchWithInternetPackageName.value),
                AppData(
                    nonLaunchWithInternetPackageName,
                    0,
                    nonLaunchWithInternetPackageName.value,
                    true,
                ),
                AppData(
                    leanbackLaunchWithInternetPackageName,
                    0,
                    leanbackLaunchWithInternetPackageName.value,
                ),
            )

        assertLists(expected, result)

        verifyAll {
            mockedPackageManager.getInstalledApplications(PackageManager.GET_META_DATA)

            // Ensure checkPermission was invoked on all packages
            listOf(
                    launchWithInternetPackageName,
                    launchWithoutInternetPackageName,
                    nonLaunchWithInternetPackageName,
                    nonLaunchWithoutInternetPackageName,
                    leanbackLaunchWithInternetPackageName,
                    leanbackLaunchWithoutInternetPackageName,
                    selfPackageName,
                )
                .forEach { packageName ->
                    mockedPackageManager.checkPermission(internet, packageName.value)
                }

            listOf(
                    launchWithInternetPackageName,
                    nonLaunchWithInternetPackageName,
                    leanbackLaunchWithInternetPackageName,
                )
                .forEach { packageName ->
                    mockedPackageManager.getLaunchIntentForPackage(packageName.value)
                }

            listOf(nonLaunchWithInternetPackageName, leanbackLaunchWithInternetPackageName)
                .forEach { packageName ->
                    mockedPackageManager.getLeanbackLaunchIntentForPackage(packageName.value)
                }
        }
    }

    @SuppressLint("UseCheckPermission")
    @Test
    fun `apps should be returned in descending order`() {
        val packageNames = listOf("b", "d", "c", "a", "e").map { AppId(it) }

        every {
            mockedPackageManager.getInstalledApplications(PackageManager.GET_META_DATA)
        } returns packageNames.map { createApplicationInfo(it, launch = true, internet = true) }

        val actual = testSubject.apps()
        val expected = packageNames.sortedBy { it.value }.map { AppData(it, 0, it.value) }

        assertEquals(expected, actual)
    }

    private fun createApplicationInfo(
        packageName: AppId,
        launch: Boolean = false,
        leanback: Boolean = false,
        internet: Boolean = false,
        systemApp: Boolean = false,
    ): ApplicationInfo {
        val mockApplicationInfo = mockk<ApplicationInfo>()

        mockApplicationInfo.packageName = packageName.value
        mockApplicationInfo.icon = 0

        every { mockApplicationInfo.loadLabel(mockedPackageManager) } returns packageName.value

        every { mockedPackageManager.getLaunchIntentForPackage(packageName.value) } returns
            if (launch || systemApp) mockk() else null

        every { mockedPackageManager.getLeanbackLaunchIntentForPackage(packageName.value) } returns
            if (leanback || systemApp) mockk() else null

        every {
            mockedPackageManager.checkPermission(Manifest.permission.INTERNET, packageName.value)
        } returns
            if (internet) PackageManager.PERMISSION_GRANTED else PackageManager.PERMISSION_DENIED

        return mockApplicationInfo
    }
}
