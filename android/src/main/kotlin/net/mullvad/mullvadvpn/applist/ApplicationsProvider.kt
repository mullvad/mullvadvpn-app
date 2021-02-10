package net.mullvad.mullvadvpn.applist

import android.Manifest
import android.content.pm.ApplicationInfo
import android.content.pm.PackageManager
import android.util.Log
import kotlin.system.measureTimeMillis

class ApplicationsProvider(
    private val packageManager: PackageManager,
    private val thisPackageName: String
) {
    private val applicationFilterPredicate: (ApplicationInfo) -> Boolean = { appInfo ->
        hasInternetPermission(appInfo.packageName) &&
            isLaunchable(appInfo.packageName) &&
            !isSelfApplication(appInfo.packageName)
    }

    fun getAppsList(): List<AppInfo2> {
        val list = ArrayList<ApplicationInfo>()
        val fetchTime = measureTimeMillis {
            list.addAll(packageManager.getInstalledApplications(PackageManager.GET_META_DATA))
        }
        Log.d("test", "fetchtime=$fetchTime")
        val result = ArrayList<AppInfo2>()
        val tempResult = ArrayList<ApplicationInfo>()
        val filterTime = measureTimeMillis {
            tempResult.addAll(list.filter(applicationFilterPredicate))
        }
        Log.d("test", "filter=$filterTime")
        val mapTime = measureTimeMillis {
            result.addAll(
                tempResult.map { info ->
                    AppInfo2(info.packageName, info.icon, info.loadLabel(packageManager).toString())
                }
            )
        }
        Log.d("test", "mapTime=$mapTime")
        return result
//            .sortedBy { info -> info.title }
    }

    private fun hasInternetPermission(packageName: String): Boolean {
        return PackageManager.PERMISSION_GRANTED ==
            packageManager.checkPermission(Manifest.permission.INTERNET, packageName)
    }

    private fun isSelfApplication(packageName: String): Boolean {
        return packageName == thisPackageName
    }

    private fun isLaunchable(packageName: String): Boolean {
        return packageManager.getLaunchIntentForPackage(packageName) != null
    }
}
