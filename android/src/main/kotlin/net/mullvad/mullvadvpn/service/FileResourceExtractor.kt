package net.mullvad.mullvadvpn.service

import android.content.Context
import java.io.File
import java.io.FileOutputStream

class FileResourceExtractor(val asset: String) {
    fun extract(context: Context) {
        val destination = File(context.filesDir, asset)

        if (!destination.exists()) {
            extractFile(context, destination)
        }
    }

    private fun extractFile(context: Context, destination: File) {
        val destinationStream = FileOutputStream(destination)

        context
            .assets
            .open(asset)
            .copyTo(destinationStream)

        destinationStream.close()
    }
}
