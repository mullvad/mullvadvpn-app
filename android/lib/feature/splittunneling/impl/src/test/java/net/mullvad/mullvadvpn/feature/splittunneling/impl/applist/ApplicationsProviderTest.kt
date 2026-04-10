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
import net.mullvad.mullvadvpn.lib.model.PackageName
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Test

class ApplicationsProviderTest {
    private val mockedPackageManager = mockk<PackageManager>()
    private val self = PackageName("self_package_name")
    private val testSubject = ApplicationsProvider(mockedPackageManager, self)

    private val internet = Manifest.permission.INTERNET

    @AfterEach
    fun tearDown() {
        unmockkAll()
    }

    @SuppressLint("UseCheckPermission")
    @Test
    fun `fetch all apps should work`() {
        val launchWithInternet = PackageName("launch_with_internet_package_name")
        val launchWithoutInternet = PackageName("launch_without_internet_package_name")
        val nonLaunchWithInternet = PackageName("non_launch_with_internet_package_name")
        val nonLaunchWithoutInternet = PackageName("non_launch_without_internet_package_name")
        val leanbackLaunchWithInternet = PackageName("leanback_launch_with_internet_package_name")
        val leanbackLaunchWithoutInternet =
            PackageName("leanback_launch_without_internet_package_name")

        every {
            mockedPackageManager.getInstalledApplications(PackageManager.GET_META_DATA)
        } returns
            listOf(
                createApplicationInfo(launchWithInternet, launch = true, internet = true),
                createApplicationInfo(launchWithoutInternet, launch = true),
                createApplicationInfo(nonLaunchWithInternet, internet = true),
                createApplicationInfo(nonLaunchWithoutInternet),
                createApplicationInfo(leanbackLaunchWithInternet, leanback = true, internet = true),
                createApplicationInfo(leanbackLaunchWithoutInternet, leanback = true),
                createApplicationInfo(self, internet = true, launch = true),
            )

        val result = testSubject.apps()
        val expected =
            listOf(
                AppData(launchWithInternet, 0, launchWithInternet.value),
                AppData(nonLaunchWithInternet, 0, nonLaunchWithInternet.value, true),
                AppData(leanbackLaunchWithInternet, 0, leanbackLaunchWithInternet.value),
            )

        assertLists(expected, result)

        verifyAll {
            mockedPackageManager.getInstalledApplications(PackageManager.GET_META_DATA)

            // Ensure checkPermission was invoked on all packages
            listOf(
                    launchWithInternet,
                    launchWithoutInternet,
                    nonLaunchWithInternet,
                    nonLaunchWithoutInternet,
                    leanbackLaunchWithInternet,
                    leanbackLaunchWithoutInternet,
                    self,
                )
                .forEach { packageName ->
                    mockedPackageManager.checkPermission(internet, packageName.value)
                }

            listOf(launchWithInternet, nonLaunchWithInternet, leanbackLaunchWithInternet).forEach {
                packageName ->
                mockedPackageManager.getLaunchIntentForPackage(packageName.value)
            }

            listOf(nonLaunchWithInternet, leanbackLaunchWithInternet).forEach { packageName ->
                mockedPackageManager.getLeanbackLaunchIntentForPackage(packageName.value)
            }
        }
    }

    @SuppressLint("UseCheckPermission")
    @Test
    fun `apps should be returned in descending order`() {
        val packageNames = listOf("b", "d", "c", "a", "e").map { PackageName(it) }

        every {
            mockedPackageManager.getInstalledApplications(PackageManager.GET_META_DATA)
        } returns packageNames.map { createApplicationInfo(it, launch = true, internet = true) }

        val actual = testSubject.apps()
        val expected = packageNames.sortedBy { it.value }.map { AppData(it, 0, it.value) }

        assertEquals(expected, actual)
    }

    private fun createApplicationInfo(
        packageName: PackageName,
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
