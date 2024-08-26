package net.mullvad.mullvadvpn.service.util

import android.content.Context
import java.io.File
import java.io.FileOutputStream

fun Context.extractAndOverwriteIfAssetMoreRecent(assetName: String) {
    val forceOverwriteIfMoreRecent = lastUpdatedTime() > File(filesDir, assetName).lastModified()
    val destination = File(filesDir, assetName)

    if (!destination.exists() || forceOverwriteIfMoreRecent) {
        extractFile(assetName, destination)
    }
}

private fun Context.lastUpdatedTime(): Long =
    packageManager.getPackageInfo(packageName, 0).lastUpdateTime

private fun Context.extractFile(asset: String, destination: File) {
    val destinationStream = FileOutputStream(destination)
    assets.open(asset).copyTo(destinationStream)
    destinationStream.close()
}
