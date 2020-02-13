package net.mullvad.mullvadvpn.service

import android.content.Context
import java.io.File
import java.io.FileOutputStream

class FileResourceExtractor(val asset: String, val destination: String) {
    fun extract(context: Context) {
        if (!File(destination).exists()) {
            extractFile(context)
        }
    }

    private fun extractFile(context: Context) {
        val destinationStream = FileOutputStream(destination)

        context
            .assets
            .open(asset)
            .copyTo(destinationStream)

        destinationStream.close()
    }
}
