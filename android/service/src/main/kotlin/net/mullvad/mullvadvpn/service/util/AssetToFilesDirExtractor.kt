package net.mullvad.mullvadvpn.service.util

import android.content.Context
import java.io.File
import java.io.FileOutputStream

class AssetToFilesDirExtractor(val context: Context) {
    fun extract(assetName: String, overwriteFileIfAssetMoreRecent: Boolean = false) {
        val forceOverwrite =
            if (overwriteFileIfAssetMoreRecent) {
                context.lastUpdatedTime() > File(context.filesDir, assetName).lastModified()
            } else false

        val destination = File(context.filesDir, assetName)

        if (!destination.exists() || forceOverwrite) {
            extractFile(assetName, destination)
        }
    }

    private fun Context.lastUpdatedTime(): Long =
        packageManager.getPackageInfo(packageName, 0).lastUpdateTime

    private fun extractFile(asset: String, destination: File) {
        val destinationStream = FileOutputStream(destination)

        context.assets.open(asset).copyTo(destinationStream)

        destinationStream.close()
    }
}
