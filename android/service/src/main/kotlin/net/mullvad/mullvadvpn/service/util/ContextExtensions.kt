package net.mullvad.mullvadvpn.service.util

import android.content.Context
import co.touchlab.kermit.Logger
import java.io.File
import java.io.FileNotFoundException
import java.io.FileOutputStream

fun Context.extractAndOverwriteIfAssetMoreRecent(assetName: String, requireAssetFile: Boolean) {
    val forceOverwriteIfMoreRecent = lastUpdatedTime() > File(filesDir, assetName).lastModified()
    val destination = File(filesDir, assetName)

    if (!destination.exists() || forceOverwriteIfMoreRecent) {
        extractFile(assetName, destination, requireAssetFile)
    }
}

private fun Context.lastUpdatedTime(): Long =
    packageManager.getPackageInfo(packageName, 0).lastUpdateTime

private fun Context.extractFile(asset: String, destination: File, requireAssetFile: Boolean) {
    if (assets.list("")?.contains(asset) == true) {
        val destinationStream = FileOutputStream(destination)
        assets.open(asset).copyTo(destinationStream)
        destinationStream.close()
    } else if (requireAssetFile) {
        throw FileNotFoundException("Asset $asset not found")
    } else {
        Logger.i("Asset $asset not found")
    }
}
