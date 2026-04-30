package net.mullvad.mullvadvpn.feature.applisting.impl

import android.content.Context
import android.content.pm.PackageInstaller
import android.content.pm.PackageManager
import android.os.Build

class AndroidInstallSourceProvider(private val context: Context) : InstallSourceProvider {
    override fun isInstalledFromStore(): Boolean {
        val packageName = context.packageName
        val packageManager = context.packageManager
        return when {
            Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU ->
                try {
                    packageManager.getInstallSourceInfo(packageName).packageSource ==
                        PackageInstaller.PACKAGE_SOURCE_STORE
                } catch (_: PackageManager.NameNotFoundException) {
                    false
                }
            Build.VERSION.SDK_INT >= Build.VERSION_CODES.R ->
                try {
                    packageManager.getInstallSourceInfo(packageName).installingPackageName != null
                } catch (_: PackageManager.NameNotFoundException) {
                    false
                }
            else ->
                @Suppress("DEPRECATION")
                (packageManager.getInstallerPackageName(packageName) != null)
        }
    }
}
